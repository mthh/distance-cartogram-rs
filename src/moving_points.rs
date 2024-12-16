use crate::errors::Error;
use crate::utils::{distance, extrapole_line, interpolate_line, median};
use geo_types::Coord;

pub enum CentralTendency {
    Mean,
    Median,
}

pub fn move_points(
    source_points: &[Coord],
    times: &[f64],
    factor: f64,
    method: CentralTendency,
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

    let ref_speed = match method {
        CentralTendency::Mean => {
            pt_time.iter().map(|(_, _, _, speed)| speed).sum::<f64>() / pt_time.len() as f64
        }
        CentralTendency::Median => {
            let speeds = pt_time
                .iter()
                .map(|(_, _, _, speed)| *speed)
                .collect::<Vec<_>>();
            median(&speeds)
        }
    };

    // So we have (point, time, distance, speed, displacement).
    let pt_times_displacement: Vec<(&Coord, f64, f64, f64, f64)> = pt_time
        .iter()
        .map(|&(pt, t, dist, speed)| (pt, t, dist, speed, ref_speed / speed))
        .collect();

    // Reconstruction of the points (taking care of the reference point).
    let mut new_points = Vec::with_capacity(source_points.len());

    for (i, (pt, t, dist, speed, displacement)) in pt_times_displacement.into_iter().enumerate() {
        let d = 1. + (displacement - 1.) * factor;
        let new_pt = if displacement < 1.0 {
            interpolate_line(ref_point, pt, d * dist)
        } else {
            let o_pt = extrapole_line(ref_point, pt, 2. * d);
            interpolate_line(&ref_point, &o_pt, d * dist)
        };
        new_points.push(new_pt);
        if i == idx {
            new_points.push(*ref_point);
        }
    }

    Ok(new_points)
}
