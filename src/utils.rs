use geo_types::Coord;

pub(crate) fn distance_sq(p1: &Coord, p2: &Coord) -> f64 {
    (p1.x - p2.x).powi(2) + (p1.y - p2.y).powi(2)
}

/// Get the number of iterations for the interpolation
/// given the number of points to interpolate.
pub fn get_nb_iterations(nb_points: usize) -> usize {
    (4. * (nb_points as f64).sqrt()).round() as usize
}
