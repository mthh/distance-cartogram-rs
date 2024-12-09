use crate::bbox::BBox;
use crate::rectangle::Rectangle2D;
use geo_types::Coord;

pub(crate) fn get_bbox(points: &[Coord]) -> BBox {
    let mut rect = Rectangle2D::new(0.0, 0.0, 0.0, 0.0);
    for pt in points {
        rect.add(pt);
    }
    rect.as_bbox()
}

pub(crate) fn distance(p1: &Coord, p2: &Coord) -> f64 {
    ((p1.x - p2.x).powi(2) + (p1.y - p2.y).powi(2)).sqrt()
}
