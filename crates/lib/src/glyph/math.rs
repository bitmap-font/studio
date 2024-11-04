use std::{iter::once, usize};

use kurbo::Point;
use strum::FromRepr;

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Pos {
    pub r: usize,
    pub c: usize,
}

impl Pos {
    pub const MIN: Pos = Pos {
        r: usize::MIN,
        c: usize::MIN,
    };
    pub const MAX: Pos = Pos {
        r: usize::MAX,
        c: usize::MAX,
    };

    pub fn shifted(&self, direction: &Direction) -> Pos {
        match direction {
            Direction::Up => Pos {
                r: self
                    .r
                    .checked_sub(1)
                    .expect("Pos.shifted(Up) should success"),
                c: self.c,
            },
            Direction::Left => Pos {
                r: self.r,
                c: self
                    .c
                    .checked_sub(1)
                    .expect("Pos().shifted(Left) should success"),
            },
            Direction::Down => Pos {
                r: self.r + 1,
                c: self.c,
            },
            Direction::Right => Pos {
                r: self.r,
                c: self.c + 1,
            },
        }
    }

    pub fn as_kurbo_point(&self, scale: usize) -> Point {
        Point::new((self.c * scale) as f64, (self.r * scale) as f64)
    }
}

/// Nearly identical to [`core::ops::Range<Pos>`]
#[derive(Clone)]
pub struct BoundingBox {
    min: Pos,
    max: Pos,
}

impl BoundingBox {
    pub const EMPTY: BoundingBox = BoundingBox {
        min: Pos::MAX,
        max: Pos::MIN,
    };

    pub fn width(&self) -> usize {
        self.max.c - self.min.c
    }

    pub fn merge(&mut self, pos_iter: &impl IterPos) {
        for pos in pos_iter.iter() {
            self.min = Pos {
                r: self.min.r.min(pos.r),
                c: self.min.c.min(pos.c),
            };
            self.max = Pos {
                r: self.max.r.max(pos.r),
                c: self.max.c.max(pos.c),
            };
        }
    }

    pub fn iterate_intpos(&self) -> impl Iterator<Item = Pos> + '_ {
        (self.min.r..self.max.r)
            .flat_map(move |r| (self.min.c..self.max.c).map(move |c| Pos { r, c }))
    }
}

trait IterPos {
    fn iter(&self) -> impl Iterator<Item = &Pos>;
}

impl IterPos for Pos {
    fn iter(&self) -> impl Iterator<Item = &Pos> {
        once(self)
    }
}
impl IterPos for BoundingBox {
    fn iter(&self) -> impl Iterator<Item = &Pos> {
        once(&self.min).chain(once(&self.max))
    }
}

#[derive(FromRepr, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Direction {
    Up = 0,
    Right = 1,
    Down = 2,
    Left = 3,
}
impl Direction {
    pub fn rotate_cw(self) -> Direction {
        Direction::from_repr((self as u8 + 1) % 4).unwrap()
    }
    pub fn flip(self) -> Direction {
        Direction::from_repr((self as u8 + 2) % 4).unwrap()
    }
    pub fn rotate_ccw(self) -> Direction {
        Direction::from_repr((self as u8 + 3) % 4).unwrap()
    }
}

#[derive(Clone)]
pub struct Matrix2x2<T>(pub [[T; 2]; 2]);

impl<T> Matrix2x2<T> {
    /// Rotate itself clockwise like:
    ///
    /// ```txt
    ///  a   b        c   a
    ///          =>
    ///  c   d        d   b
    /// ```
    pub fn rotate_cw(self) -> Matrix2x2<T> {
        let Matrix2x2([[a, b], [c, d]]) = self;
        Matrix2x2([[c, a], [d, b]])
    }

    /// Rotate itself counterclockwise like:
    ///
    /// ```txt
    ///  a   b        b   d
    ///          =>
    ///  c   d        a   c
    /// ```
    pub fn rotate_ccw(self) -> Matrix2x2<T> {
        let Matrix2x2([[a, b], [c, d]]) = self;
        Matrix2x2([[b, d], [a, c]])
    }

    /// Flip itself like:
    ///
    /// ```txt
    ///  a   b        d   c
    ///          =>
    ///  c   d        b   a
    /// ```
    pub fn flip(self) -> Matrix2x2<T> {
        let Matrix2x2([[a, b], [c, d]]) = self;
        Matrix2x2([[d, c], [b, a]])
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.0.iter().flatten()
    }

    pub fn map<U>(self, mut f: impl FnMut(T) -> U) -> Matrix2x2<U> {
        Matrix2x2(self.0.map(move |arr| arr.map(|v| f(v))))
    }
}
