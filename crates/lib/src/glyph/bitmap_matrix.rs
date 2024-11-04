use std::collections::HashSet;

use kurbo::{BezPath, Point};
use yaff::{GlyphDefinition, GlyphPaletteColor};

use crate::glyph::{
    math::Matrix2x2,
    pathfinder::{find_path, MonochromeField},
};

use super::math::{BoundingBox, Pos};

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

    pub fn as_bezier_paths(&self, scale: usize) -> (Vec<BezPath>, BoundingBox) {
        struct Field<'a> {
            mat: &'a BitmapMatrix,
            color: &'a GlyphPaletteColor,
        }
        impl MonochromeField for Field<'_> {
            fn is_colored_of_truthy_pos(&self, r: usize, c: usize) -> bool {
                self.mat
                    .0
                    .get(r)
                    .and_then(|row| row.get(c))
                    .map_or(false, |v| matches!(v, Some(v) if v == self.color))
            }
        }

        let height = self.0.len();
        let width = self.0.get(0).map(Vec::len).unwrap_or(0);
        let dots = Vec::from_iter((0..height).flat_map(|r| (0..width).map(move |c| Pos { r, c })));

        let mut result = Vec::new();
        let mut whole_bb = BoundingBox::EMPTY;

        let mut consumed_dots = HashSet::new();
        for pos in dots {
            if consumed_dots.contains(&pos) {
                continue;
            }
            let Some(color) = &self.0[pos.r][pos.c] else {
                consumed_dots.insert(pos);
                continue;
            };

            let (path, path_bb) = find_path(pos, scale, Field { mat: &self, color }, |pos| {
                consumed_dots.insert(pos);
            });

            result.push(path);
            whole_bb.merge(&path_bb);
        }

        (result, whole_bb)
    }
}
