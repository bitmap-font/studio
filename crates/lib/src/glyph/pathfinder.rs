use kurbo::BezPath;

use super::math::{Direction, Matrix2x2, Pos};

const IS_DEBUG: bool = false;

#[derive(PartialEq, Eq)]
pub enum PathfinderMode {
    Contour,
    Hole,
}

pub fn find_path(
    begin: Pos,
    scale: usize,
    field: impl MonochromeField,
    consumption_reporter: impl FnMut(Pos) -> (),
) -> BezPath {
    let mut path = BezPath::new();
    _find_path(
        begin,
        scale,
        field,
        consumption_reporter,
        PathfinderMode::Contour,
        &mut path,
    );

    path
}

fn _find_path(
    begin_l: Pos,
    scale: usize,
    field: impl MonochromeField,
    mut consumption_reporter: impl FnMut(Pos) -> (),
    mode: PathfinderMode,
    path: &mut BezPath,
) {
    let begin_r = begin_l.shifted(&Direction::Right);
    // we use top line of the pixel for the start of contour travelling
    //   @====>            <====@
    //   |    |     OR     |    |
    //   |    |            |    |
    //   +----+            +----+
    // Note that hole represented by its direction (ccw)
    let (actual_begin, mut pos, mut direction) = if mode == PathfinderMode::Contour {
        path.move_to(begin_l.as_kurbo_point(scale));
        (begin_l, begin_r, Direction::Right)
    } else {
        path.move_to(begin_r.as_kurbo_point(scale));
        (begin_r, begin_l, Direction::Left)
    };
    let is_contour = mode == PathfinderMode::Contour;
    let mut size = 1;

    if IS_DEBUG {
        eprintln!("{actual_begin:?} -> {pos:?} : {direction:?}");
    }

    while actual_begin != pos {
        consumption_reporter(pos.clone());

        let mat = Matrix2x2([
            [
                field.is_colored(pos.r.checked_sub(1), pos.c.checked_sub(1)),
                field.is_colored(pos.r.checked_sub(1), pos.c),
            ],
            [
                field.is_colored(pos.r, pos.c.checked_sub(1)),
                field.is_colored(pos.r, pos.c),
            ],
        ]);
        if is_contour {
            for pos in mat.iter().flatten() {
                consumption_reporter(pos.clone());
            }
        }
        let mat = mat.map(|x| x.is_some() == is_contour);

        let next_direction = match direction {
            Direction::Up => next_direction(&mat.clone().rotate_cw()).map(Direction::rotate_ccw),
            Direction::Left => next_direction(&mat.clone().flip()).map(Direction::flip),
            Direction::Down => next_direction(&mat.clone().rotate_ccw()).map(Direction::rotate_cw),
            Direction::Right => next_direction(&mat),
        };

        if IS_DEBUG {
            _debug_flow(&mat, is_contour, &pos, &direction, &next_direction);
        }

        let Some(next_direction) = next_direction else {
            panic!("next direction must be decided");
        };
        if direction != next_direction {
            path.line_to(pos.as_kurbo_point(scale));
            size = 0;
        }
        size += 1;
        pos = pos.shifted(&next_direction);
        direction = next_direction;
    }
    if size != 0 {
        path.line_to(pos.as_kurbo_point(scale));
    }
    path.close_path();
}

fn _debug_flow(
    Matrix2x2([[lt, rt], [lb, rb]]): &Matrix2x2<bool>,
    is_contour: bool,
    pos: &Pos,
    from_dir: &Direction,
    to_dir: &Option<Direction>,
) {
    println!(
        "{pos:?} : {from_dir:?} to {}",
        to_dir
            .as_ref()
            .map_or_else(|| "???".to_string(), |x| format!("{:?}", x))
    );
    let vline = match (from_dir, to_dir) {
        (Direction::Down, _) => Some("v"),
        (_, Some(Direction::Up)) => Some("^"),
        (_, _) => None,
    };
    println!("  +-----{vline}-----+", vline = vline.unwrap_or("+"));
    println!(
        "  | {lt_top} {vline} {rt_top} |",
        lt_top = if *lt { "___" } else { "   " },
        vline = vline.unwrap_or("|"),
        rt_top = if *rt { "___" } else { "   " }
    );
    println!(
        "  | {lt_body} {vline} {rt_body} |",
        lt_body = if *lt { "@@@" } else { "   " },
        vline = vline.unwrap_or("|"),
        rt_body = if *rt { "@@@" } else { "   " }
    );
    println!(
        "  | {lt_bottom} {vline} {rt_bottom} |",
        lt_bottom = if *lt { "***" } else { "   " },
        vline = vline.unwrap_or("|"),
        rt_bottom = if *rt { "***" } else { "   " }
    );
    println!(
        "  {}",
        match (from_dir, to_dir) {
            (Direction::Right, Some(Direction::Right)) => "> > > > > > >",
            (Direction::Right, Some(Direction::Up)) => "> > > /-----+",
            (Direction::Right, Some(Direction::Down)) => "> > > \\-----+",
            (Direction::Right, _) => "> > > ?-----+",
            (Direction::Left, Some(Direction::Left)) => "< < < < < < <",
            (Direction::Left, Some(Direction::Up)) => "+-----\\ < < <",
            (Direction::Left, Some(Direction::Down)) => "+-----/ < < <",
            (Direction::Left, _) => "+-----? < < <",
            (Direction::Up, Some(Direction::Left)) => "< < < \\-----+",
            (Direction::Down, Some(Direction::Left)) => "< < < /-----+",
            (Direction::Up, Some(Direction::Right)) => "+-----/ > > >",
            (Direction::Down, Some(Direction::Right)) => "+-----\\ > > >",
            (_, None) => "+---- ? ----+",
            (_, _) => "+-----+-----+",
        }
    );
    let vline = match (from_dir, to_dir) {
        (Direction::Up, _) => Some("^"),
        (_, Some(Direction::Down)) => Some("v"),
        (_, _) => None,
    };
    println!(
        "  | {lb_top} {vline} {rb_top} |",
        lb_top = if *lb { "___" } else { "   " },
        vline = vline.unwrap_or("|"),
        rb_top = if *rb { "___" } else { "   " },
    );
    println!(
        "  | {lb_body} {vline} {rb_body} |",
        lb_body = if *lb { "@@@" } else { "   " },
        vline = vline.unwrap_or("|"),
        rb_body = if *rb { "@@@" } else { "   " }
    );
    println!(
        "  | {lb_bottom} {vline} {rb_bottom} |",
        lb_bottom = if *lb { "***" } else { "   " },
        vline = vline.unwrap_or("|"),
        rb_bottom = if *rb { "***" } else { "   " }
    );
    println!("  +-----{vline}-----+", vline = vline.unwrap_or("+"));
}

/// In this situation, this function chooses what direction should we go
/// ```text
/// +----+----+
/// | lt | rt |
/// @====>----+
/// | lb | rb |
/// +----+----+
/// ``````
fn next_direction(Matrix2x2([[lt, rt], [lb, rb]]): &Matrix2x2<bool>) -> Option<Direction> {
    match (lt, rt, lb, rb) {
        (false, _, false, _) | (true, _, true, _) => {
            None
            // panic!("lt xor lb must be true but (lt={lt}, lb={lb})")
        }

        //
        //                   @ ?
        // --+       OR     --+
        //  @|                |@
        //   v                v
        (false, false, true, false) | (true, _, false, true) => Some(Direction::Down),

        //
        //  @ @
        // ---->     OR     ---->
        //                   @ @
        //
        (true, true, false, false) | (false, false, true, true) => Some(Direction::Right),

        //   ^                ^
        //  @|                |@
        // --+       OR     --+
        //                   @ ?
        //
        (true, false, false, false) | (false, true, true, _) => Some(Direction::Up),
    }
}

pub trait MonochromeField {
    /**
     * Returns true if (r, c) is colored.
     */
    fn is_colored_of_truthy_pos(&self, r: usize, c: usize) -> bool;

    /// Helper for optional position.
    /// if both r and c is some, it's same as [`Self::check_truthy_position`].
    /// but returns Some(Pos { r, c }) for true, None for false or other cases.
    fn is_colored(
        &self,
        r: impl priv_trait::OptionableUsize,
        c: impl priv_trait::OptionableUsize,
    ) -> Option<Pos> {
        let r = r.into_option_usize()?;
        let c = c.into_option_usize()?;
        self.is_colored_of_truthy_pos(r, c).then(|| Pos { r, c })
    }
}

mod priv_trait {
    pub trait OptionableUsize {
        fn into_option_usize(self) -> Option<usize>;
    }
    impl OptionableUsize for Option<usize> {
        fn into_option_usize(self) -> Option<usize> {
            self
        }
    }
    impl OptionableUsize for usize {
        fn into_option_usize(self) -> Option<usize> {
            Some(self)
        }
    }
}
