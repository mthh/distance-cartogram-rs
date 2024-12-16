use crate::errors::Error;
use crate::utils::{distance, interpolate_line};
use geo_types::Coord;

pub fn move_points(
    source_points: &[Coord],
    times: &[f64],
    factor: f64,
) -> Result<Vec<Coord>, Error> {
    if source_points.len() != times.len() {
        return Err(Error::InvalidInputTimesLength);
    }
    // Find the index for which the time is 0,
    // this will be our reference points for the movement
    // of the other points.
    // If there is none, this is an error and we return.
    let idx = times
        .iter()
        .position(|&t| t == 0.0)
        .ok_or(Error::NoReferencePoint)?;

    let ref_point = &source_points[idx];
    // Get all the points that are not the reference point
    // associated with their time.
    // So we have (point, time, distance, speed).
    let pt_time: Vec<(&Coord, f64, f64, f64)> = source_points
        .iter()
        .zip(times.iter())
        .filter(|(_, &t)| t != 0.0)
        .map(|(pt, &t)| {
            let dist = distance(ref_point, pt);
            (pt, t, dist, dist / t)
        })
        .collect();

    let ref_speed = pt_time
        .iter()
        .fold(0.0, |acc, &(_, _, _, speed)| acc + speed)
        / (pt_time.len() - 1) as f64;

    // So we have (point, time, distance, speed, displacement).
    let pt_times_displacement: Vec<(&Coord, f64, f64, f64, f64)> = pt_time
        .iter()
        .map(|&(pt, t, dist, speed)| (pt, t, dist, speed, ref_speed / speed))
        .collect();

    // Reconstruction of the points (taking care of the reference point).
    let mut new_points = Vec::with_capacity(source_points.len());

    for (i, (pt, t, dist, speed, displacement)) in pt_times_displacement.into_iter().enumerate() {
        let displacement = 1. + (displacement - 1.) * factor;
        let new_pt = interpolate_line(ref_point, pt, displacement);
        new_points.push(new_pt);
        if i == idx {
            new_points.push(*ref_point);
        }
    }

    Ok(new_points)
}
