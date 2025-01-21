use crate::errors::Error;
use crate::utils::{buffer_around_point, distance, interpolate_line, median};
use geo_types::Coord;

/// The central tendency method to use to compute the reference speed
/// for the movement of the points in the [`move_points`] function.
pub enum CentralTendency {
    Mean,
    Median,
}

/// The result of the movement of the points.
pub struct MovePointsResult {
    /// The moved points.
    pub points: Vec<Coord>,
    /// The reference speed used for the movement
    /// (can be useful to create concentric circles
    /// around the reference point).
    pub reference_speed: f64,
    /// The reference point used for the movement.
    pub reference_point: Coord,
}

/// Move the points (using a central tendency method such as the
/// mean or the median to compute the reference speed and determine
/// how the points are moved).
///
/// If the points can can be reached with a speed
/// (Euclidean distance / time) larger than the reference speed,
/// they will be moved further away and conversely if they can be
/// reached with a speed smaller than the reference speed, they will
/// be moved closer.
///
/// The factor is a multiplier to the displacement that is computed
/// from the reference speed (a factor of 1.0 should be the default
/// value to use and a larger factor will move the points further
/// away).
///
/// Note that the source points and the durations must have the same length
/// and that there must be a reference point for which the duration is 0.
/// If one of these conditions is not met, an error is returned.
pub fn move_points(
    source_points: &[Coord],
    durations: &[f64],
    factor: f64,
    method: CentralTendency,
) -> Result<MovePointsResult, Error> {
    if source_points.len() != durations.len() {
        return Err(Error::InvalidInputDurationsLength);
    }
    // Find the index for which the duration is 0,
    // this will be our reference points for the movement
    // of the other points.
    // If there is none, this is an error and we return.
    let idx = durations
        .iter()
        .position(|&t| t == 0.0)
        .ok_or(Error::NoReferencePoint)?;

    let ref_point = &source_points[idx];
    // Get all the points that are not the reference point
    // associated with their duration.
    // So we have (point, duration, distance, speed).
    let pt_time: Vec<(&Coord, f64, f64, f64)> = source_points
        .iter()
        .zip(durations.iter())
        .filter(|(_, &t)| t != 0.0)
        .map(|(pt, &t)| {
            let dist = distance(ref_point, pt);
            (pt, t, dist, dist / t)
        })
        .collect();

    // Compute the reference speed from the given central tendency method
    let ref_speed = match method {
        CentralTendency::Mean => {
            pt_time.iter().map(|(_, _, _, speed)| speed).sum::<f64>() / pt_time.len() as f64
        }
        CentralTendency::Median => {
            let speeds = pt_time
                .iter()
                .map(|(_, _, _, speed)| *speed)
                .collect::<Vec<_>>();
            median(speeds)
        }
    };

    // Get the displacement factor for each point given the reference speed.
    // So we have (point, duration, distance, speed, displacement).
    let pt_times_displacement: Vec<(&Coord, f64, f64, f64, f64)> = pt_time
        .iter()
        .map(|&(pt, d, dist, speed)| (pt, d, dist, speed, ref_speed / speed))
        .collect();

    // Reconstruction of the points (taking care of the reference point).
    let mut new_points = Vec::with_capacity(source_points.len());

    for (pt, _d, dist, _speed, displacement) in pt_times_displacement.into_iter() {
        // Combine the factor and the computed displacement value
        let d = 1. + (displacement - 1.) * factor;
        // Actually compute the position of the moved point
        new_points.push(interpolate_line(ref_point, pt, d * dist));
    }

    // Add the reference point at the right index
    new_points.insert(idx, *ref_point);

    Ok(MovePointsResult {
        points: new_points,
        reference_point: *ref_point,
        reference_speed: ref_speed,
    })
}

/// Takes the result of the unipolar movement of the points and creates
/// concentric circles (as LineStrings), at the given steps, around the
/// reference point.
///
/// The steps are the durations at which the circles will be created
/// (in the unit of the duration between the reference point and the
/// other points).
pub fn concentric_circles(
    move_points_result: &MovePointsResult,
    steps: Vec<f64>,
) -> Vec<(geo_types::Geometry, f64)> {
    let ref_point = move_points_result.reference_point;
    let ref_speed = move_points_result.reference_speed;
    let mut circles = Vec::with_capacity(steps.len());

    for step in steps {
        let circle = buffer_around_point(&ref_point, ref_speed * step, 100);
        circles.push((geo_types::Geometry::from(circle), step));
    }

    circles
}
