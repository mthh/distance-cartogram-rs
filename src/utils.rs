use crate::grid::RMSE;
use geo_types::Coord;

pub(crate) fn distance_sq(p1: &Coord, p2: &Coord) -> f64 {
    (p1.x - p2.x).powi(2) + (p1.y - p2.y).powi(2)
}

#[cfg(feature = "moving-points-unipolar")]
pub(crate) fn distance(p1: &Coord, p2: &Coord) -> f64 {
    distance_sq(p1, p2).sqrt()
}

/// Get the number of iterations for the interpolation
/// given the number of points to interpolate.
pub fn get_nb_iterations(nb_points: usize) -> usize {
    (4. * (nb_points as f64).sqrt()).round() as usize
}

/// Compute the Root Mean Square Error (RMSE).
/// It usually measures differences between predicted values and observed values
/// and gives an idea of the overall accuracy of the regression.
pub(crate) fn rmse(points1: &[Coord], points2: &[Coord]) -> RMSE {
    let n = points1.len();
    let nf = n as f64;
    let mut sum_sq_error_x = 0.0;
    let mut sum_sq_error_y = 0.0;
    for i in 0..n {
        let dx = points1[i].x - points2[i].x;
        let dy = points1[i].y - points2[i].y;
        sum_sq_error_x += dx * dx;
        sum_sq_error_y += dy * dy;
    }
    RMSE {
        rmse: ((sum_sq_error_x + sum_sq_error_y) / nf).sqrt(),
        rmse_x: (sum_sq_error_x / nf).sqrt(),
        rmse_y: (sum_sq_error_y / nf).sqrt(),
    }
}

/// Compute the R-squared value. It measures the proportion of the variance
/// in the dependent variable that is predictable from the independent variable(s).
/// It provides an indication of the goodness of fit of the points to the grid.
pub(crate) fn r_squared(image_points: &[Coord], interpolated_points: &[Coord]) -> f64 {
    let mut ss_total = 0.0;
    let mut ss_residual = 0.0;
    let n = image_points.len();
    let mean_x = image_points.iter().map(|p| p.x).sum::<f64>() / n as f64;
    let mean_y = image_points.iter().map(|p| p.y).sum::<f64>() / n as f64;

    for i in 0..n {
        let dx = image_points[i].x - interpolated_points[i].x;
        let dy = image_points[i].y - interpolated_points[i].y;
        ss_residual += dx * dx + dy * dy;

        let dx_total = image_points[i].x - mean_x;
        let dy_total = image_points[i].y - mean_y;
        ss_total += dx_total * dx_total + dy_total * dy_total;
    }

    1.0 - (ss_residual / ss_total)
}

/// Compute the Mean Absolute Error (MAE).
/// It measures the average magnitude of the errors in a set of predictions,
/// without considering their direction.
pub(crate) fn mae(image_points: &[Coord], interpolated_points: &[Coord]) -> f64 {
    let mut sum_abs_error = 0.0;
    let n = image_points.len();
    for i in 0..n {
        let dx = (image_points[i].x - interpolated_points[i].x).abs();
        let dy = (image_points[i].y - interpolated_points[i].y).abs();
        sum_abs_error += dx + dy;
    }
    sum_abs_error / n as f64
}

#[cfg(feature = "moving-points-unipolar")]
pub(crate) fn interpolate_line(p1: &Coord, p2: &Coord, distance_along_line: f64) -> Coord {
    let total_distance = distance(p1, p2);
    if total_distance == 0. {
        return *p1;
    }
    if total_distance == distance_along_line {
        return *p2;
    }
    let t = distance_along_line / total_distance;
    Coord {
        x: p1.x + (p2.x - p1.x) * t,
        y: p1.y + (p2.y - p1.y) * t,
    }
}

#[cfg(feature = "moving-points-unipolar")]
pub(crate) fn median(mut series: Vec<f64>) -> f64 {
    series.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let mid = series.len() / 2;
    if series.len() % 2 == 0 {
        (series[mid - 1] + series[mid]) / 2.
    } else {
        series[mid]
    }
}

#[cfg(feature = "moving-points-multipolar")]
/// Read a CSV file containing a duration matrix (so the first line is the header
/// and the first column is the row names). The header and the row names have to be
/// identical.
/// The function returns a tuple containing the matrix and the row names.
///
/// An example of a valid CSV file for this function is:
/// ```
/// ,A,B,C
/// A,0,1,2
/// B,1,0,3
/// C,2,3,0
/// ```
pub fn read_csv(file: std::fs::File) -> (Vec<Vec<f64>>, Vec<String>) {
    let mut rdr = csv::Reader::from_reader(file);
    let headers = rdr.headers().unwrap();
    let headers = headers
        .iter()
        .skip(1)
        .map(|x| x.to_string())
        .collect::<Vec<String>>();
    let mut data = Vec::new();
    for result in rdr.records() {
        let record = result.unwrap();
        let row: Vec<f64> = record
            .iter()
            .skip(1)
            .map(|x| x.parse::<f64>().unwrap())
            .collect();
        data.push(row);
    }
    (data, headers)
}
