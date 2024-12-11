use geo_types::Coord;

pub(crate) fn distance_sq(p1: &Coord, p2: &Coord) -> f64 {
    (p1.x - p2.x).powi(2) + (p1.y - p2.y).powi(2)
}
