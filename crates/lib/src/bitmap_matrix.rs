use std::collections::HashSet;

use kurbo::{BezPath, Point};
use strum::FromRepr;
use yaff::{GlyphDefinition, GlyphPaletteColor};

const DEBUG: bool = false;

pub struct BitmapMatrix(pub Vec<Vec<Option<GlyphPaletteColor>>>);

impl From<GlyphDefinition> for BitmapMatrix {
    fn from(value: GlyphDefinition) -> Self {
        let value = value.value.map(|value| value.data).unwrap_or_default();
        BitmapMatrix(value)
    }
}

impl From<&'_ GlyphDefinition> for BitmapMatrix {
    fn from(value: &GlyphDefinition) -> Self {
        let value = value
            .value
            .as_ref()
            .map(|value| value.data.clone())
            .unwrap_or_default();
        BitmapMatrix(value)
    }
}

impl BitmapMatrix {
    pub fn union(list: impl IntoIterator<Item = BitmapMatrix>) -> BitmapMatrix {
        let mut this = Vec::new();

        for BitmapMatrix(other) in list {
            while this.len() < other.len() {
                this.push(Vec::new());
            }
            for (r, row) in other.into_iter().enumerate() {
                while this[r].len() < row.len() {
                    this[r].push(None);
                }
                for (c, col) in row.into_iter().enumerate() {
                    let Some(col) = col else { continue };
                    this[r][c].replace(col);
                }
            }
        }

        BitmapMatrix(this)
    }

    pub fn as_bezier_paths(&self) -> Vec<BezPath> {
        #[derive(FromRepr, Debug, PartialEq, Eq)]
        #[repr(u8)]
        enum Direction {
            Up = 0,
            Left = 1,
            Down = 2,
            Right = 3,
        }
        impl Direction {
            fn rotate_right(self) -> Direction {
                Direction::from_repr((self as u8 + 1) % 4).unwrap()
            }
            fn rotate_left(self) -> Direction {
                Direction::from_repr((self as u8 + 3) % 4).unwrap()
            }
            fn apply(&self, pos: &mut Pos) {
                match self {
                    Direction::Up => pos.r -= 1,
                    Direction::Left => pos.c -= 1,
                    Direction::Down => pos.r += 1,
                    Direction::Right => pos.c += 1,
                }
            }
        }
        /// In this situation, this function chooses what direction should we go
        /// ```text
        /// +----+----+
        /// | lt | rt |
        /// @====>----+
        /// | lb | rb |
        /// +----+----+
        /// ``````
        fn next_direction(lt: bool, rt: bool, lb: bool, rb: bool) -> Option<Direction> {
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

        let height = self.0.len();
        let width = self.0.get(0).map(Vec::len).unwrap_or(0);
        let dots = Vec::from_iter((0..height).flat_map(|r| (0..width).map(move |c| (r, c))));

        let mut result = Vec::new();

        let mut consumed_dots = HashSet::new();
        for (r, c) in dots {
            if consumed_dots.contains(&(r, c)) {
                continue;
            }
            let Some(color) = &self.0[r][c] else {
                consumed_dots.insert((r, c));
                continue;
            };

            let mut path = BezPath::new();
            path.move_to(Point::new(c as f64, r as f64));

            let get_color = |r: Option<usize>, c: Option<usize>| -> Option<&GlyphPaletteColor> {
                self.0.get(r?)?.get(c?)?.as_ref()
            };
            let get_color_match = |r: Option<usize>, c: Option<usize>| -> bool {
                get_color(r, c).map(|x| x == color).unwrap_or(false)
            };

            #[derive(Debug)]
            struct State {
                pos: Pos,
                vec: (Direction, usize),
            }
            let mut state = State {
                pos: {
                    let mut pos = Pos { r, c };
                    Direction::Right.apply(&mut pos);
                    pos
                },
                vec: (Direction::Right, 1),
            };

            if DEBUG {
                println!("begins from ({}, {})", r, c);
            }

            while r != state.pos.r || c != state.pos.c {
                consumed_dots.insert((state.pos.r, state.pos.c));

                let mat = [
                    get_color_match(state.pos.r.checked_sub(1), state.pos.c.checked_sub(1)),
                    get_color_match(state.pos.r.checked_sub(1), Some(state.pos.c)),
                    get_color_match(Some(state.pos.r), state.pos.c.checked_sub(1)),
                    get_color_match(Some(state.pos.r), Some(state.pos.c)),
                ];
                if mat[0] {
                    consumed_dots.insert((state.pos.r - 1, state.pos.c - 1));
                }
                if mat[1] {
                    consumed_dots.insert((state.pos.r - 1, state.pos.c));
                }
                if mat[2] {
                    consumed_dots.insert((state.pos.r, state.pos.c - 1));
                }
                if mat[3] {
                    consumed_dots.insert((state.pos.r, state.pos.c));
                }

                let new_direction =
                    match state.vec.0 {
                        Direction::Up => next_direction(mat[2], mat[0], mat[3], mat[1])
                            .map(Direction::rotate_right),
                        Direction::Left => next_direction(mat[3], mat[2], mat[1], mat[0])
                            .map(Direction::rotate_left)
                            .map(Direction::rotate_left),
                        Direction::Down => next_direction(mat[1], mat[3], mat[0], mat[2])
                            .map(Direction::rotate_left),
                        Direction::Right => next_direction(mat[0], mat[1], mat[2], mat[3]),
                    };

                if DEBUG {
                    println!(
                        "({}, {}) : {:?} to {}",
                        state.pos.c,
                        state.pos.r,
                        state.vec.0,
                        new_direction
                            .as_ref()
                            .map_or_else(|| "???".to_string(), |x| format!("{:?}", x))
                    );
                    let vline = match (&state.vec.0, &new_direction) {
                        (Direction::Down, _) => Some("v"),
                        (_, Some(Direction::Up)) => Some("^"),
                        (_, _) => None,
                    };
                    println!("  +-----{vline}-----+", vline = vline.unwrap_or("+"));
                    println!(
                        "  | {lt_top} {vline} {rt_top} |",
                        lt_top = if mat[0] { "___" } else { "   " },
                        vline = vline.unwrap_or("|"),
                        rt_top = if mat[1] { "___" } else { "   " }
                    );
                    println!(
                        "  | {lt_body} {vline} {rt_body} |",
                        lt_body = if mat[0] { "@@@" } else { "   " },
                        vline = vline.unwrap_or("|"),
                        rt_body = if mat[1] { "@@@" } else { "   " }
                    );
                    println!(
                        "  | {lt_bottom} {vline} {rt_bottom} |",
                        lt_bottom = if mat[0] { "***" } else { "   " },
                        vline = vline.unwrap_or("|"),
                        rt_bottom = if mat[1] { "***" } else { "   " }
                    );
                    println!(
                        "  {}",
                        match (&state.vec.0, &new_direction) {
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
                    let vline = match (&state.vec.0, &new_direction) {
                        (Direction::Up, _) => Some("^"),
                        (_, Some(Direction::Down)) => Some("v"),
                        (_, _) => None,
                    };
                    println!(
                        "  | {lb_top} {vline} {rb_top} |",
                        lb_top = if mat[2] { "___" } else { "   " },
                        vline = vline.unwrap_or("|"),
                        rb_top = if mat[3] { "___" } else { "   " },
                    );
                    println!(
                        "  | {lb_body} {vline} {rb_body} |",
                        lb_body = if mat[2] { "@@@" } else { "   " },
                        vline = vline.unwrap_or("|"),
                        rb_body = if mat[3] { "@@@" } else { "   " }
                    );
                    println!(
                        "  | {lb_bottom} {vline} {rb_bottom} |",
                        lb_bottom = if mat[2] { "***" } else { "   " },
                        vline = vline.unwrap_or("|"),
                        rb_bottom = if mat[3] { "***" } else { "   " }
                    );
                    println!("  +-----{vline}-----+", vline = vline.unwrap_or("+"));
                }

                let Some(new_direction) = new_direction else {
                    panic!();
                };

                new_direction.apply(&mut state.pos);
                let len = if state.vec.0 == new_direction {
                    state.vec.1 + 1
                } else {
                    1
                };
                state.vec = (new_direction, len);
                path.line_to(Point::new(state.pos.c as f64, state.pos.r as f64));
            }

            path.close_path();
            result.push(path);
        }

        result
    }
}

#[derive(Debug)]
struct Pos {
    r: usize,
    c: usize,
}
