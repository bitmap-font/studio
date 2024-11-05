use std::usize;

use kurbo::Point;
use strum::FromRepr;

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Pos {
    pub r: usize,
    pub c: usize,
}

impl Pos {
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
