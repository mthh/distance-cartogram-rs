use crate::bbox::BBox;
use crate::point::Point;
use crate::rectangle::Rectangle2D;

pub(crate) fn extrapole_line(p1: (f64, f64), p2: (f64, f64), ratio: f64) -> [(f64, f64); 2] {
    let x1 = p1.0;
    let y1 = p1.1;
    let x2 = p2.0;
    let y2 = p2.1;
    let x3 = x1 + (x2 - x1) * ratio;
    let y3 = y1 + (y2 - y1) * ratio;
    [(x1, y1), (x3, y3)]
}

pub(crate) fn get_bbox(points: &[Point]) -> BBox {
    let mut rect = Rectangle2D::new(0.0, 0.0, 0.0, 0.0);
    for pt in points {
        rect.add(pt);
    }
    rect.as_bbox()
}
