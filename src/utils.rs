use geo_types::Coord;

pub(crate) fn distance_sq(p1: &Coord, p2: &Coord) -> f64 {
    (p1.x - p2.x).powi(2) + (p1.y - p2.y).powi(2)
}

#[cfg(feature = "moving-points")]
pub(crate) fn distance(p1: &Coord, p2: &Coord) -> f64 {
    distance_sq(p1, p2).sqrt()
}

/// Get the number of iterations for the interpolation
/// given the number of points to interpolate.
pub fn get_nb_iterations(nb_points: usize) -> usize {
    (4. * (nb_points as f64).sqrt()).round() as usize
}

#[cfg(feature = "moving-points")]
pub(crate) fn extrapole_line(p1: &Coord, p2: &Coord, ratio: f64) -> Coord {
    let x = p1.x + (p2.x - p1.x) * ratio;
    let y = p1.y + (p2.y - p1.y) * ratio;
    Coord { x, y }
}

#[cfg(feature = "moving-points")]
pub(crate) fn interpolate_line(p1: &Coord, p2: &Coord, distance_along_line: f64) -> Coord {
    let total_distance = distance(p1, p2);
    if total_distance == 0. {
        return *p1;
    }
    if total_distance == distance_along_line {
        return *p2;
    }
    let t = distance_along_line / total_distance;
    let x = p1.x + (p2.x - p1.x) * t;
    let y = p1.y + (p2.y - p1.y) * t;
    Coord { x, y }
}

#[cfg(feature = "moving-points")]
pub(crate) fn median(mut series: Vec<f64>) -> f64 {
    series.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let mid = series.len() / 2;
    if series.len() % 2 == 0 {
        (series[mid - 1] + series[mid]) / 2.
    } else {
        series[mid]
    }
}
