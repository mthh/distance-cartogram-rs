use crate::errors::Error;
use crate::procrustes;
use pcoa::{apply_pcoa, nalgebra::DMatrix};

/// Takes a duration matrix and a set of reference points and returns the coordinates of the points
/// derived from the PCoA analysis of the duration matrix, rotated, scaled and translated to best
/// fit the reference points using the Procrustes analysis.
pub fn generate_positions_from_durations(
    durations: Vec<Vec<f64>>,
    reference_points: &[geo_types::Coord<f64>],
) -> Result<Vec<geo_types::Coord<f64>>, Error> {
    let m = durations.len();
    for item in durations.iter() {
        if item.len() != m {
            return Err(Error::DurationMatrixNotSquare);
        }
    }
    if durations.len() != reference_points.len() {
        return Err(Error::InvalidInputDurationsDimensions);
    }
    let n_dims = 2;
    let flat_mat = durations
        .iter()
        .flat_map(|x| x.iter().copied())
        .collect::<Vec<_>>();
    let distance_matrix = DMatrix::from_column_slice(m, m, &flat_mat);
    let coords_matrix = apply_pcoa(distance_matrix, n_dims).ok_or(Error::PCoAUnsuccessful)?;
    let coords_matrix = coords_matrix.transpose();
    let xs: Vec<_> = coords_matrix.column(0).iter().copied().collect();
    let ys: Vec<_> = coords_matrix.column(1).iter().copied().collect();
    let points_target = xs
        .iter()
        .zip(ys.iter())
        .map(|(x, y)| geo_types::Coord { x: *x, y: *y })
        .collect::<Vec<_>>();
    let proc_res = procrustes::procrustes(reference_points, &points_target)?;
    Ok(proc_res.points)
}
