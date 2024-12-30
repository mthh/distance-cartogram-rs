use crate::errors::Error;
use crate::utils::{distance, interpolate_line, median};
use geo_types::Coord;

/// The central tendency method to use to compute the reference speed
/// for the movement of the points in the [`move_points`] function.
pub enum CentralTendency {
    Mean,
    Median,
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
) -> Result<Vec<Coord>, Error> {
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

    Ok(new_points)
}
