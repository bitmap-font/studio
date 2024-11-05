use std::{
    collections::HashSet,
    hash::{DefaultHasher, Hasher},
};

use kurbo::BezPath;

pub struct PointAndContours {
    pub points: usize,
    pub contours: usize,
}

pub fn analyze_bezpath(path: &BezPath) -> PointAndContours {
    let points = HashSet::<u64>::from_iter(
        path.elements()
            .iter()
            .flat_map(|el| match el {
                kurbo::PathEl::MoveTo(p1) => vec![p1],
                kurbo::PathEl::LineTo(p1) => vec![p1],
                kurbo::PathEl::QuadTo(p1, p2) => vec![p1, p2],
                kurbo::PathEl::CurveTo(p1, p2, p3) => vec![p1, p2, p3],
                kurbo::PathEl::ClosePath => vec![],
            })
            .map(|p| {
                let mut hasher = DefaultHasher::new();
                hasher.write(&p.x.to_be_bytes());
                hasher.write(&p.y.to_be_bytes());
                hasher.finish()
            }),
    );
    let points = points.len();

    let contours = path.elements().len();
    PointAndContours { points, contours }
}
