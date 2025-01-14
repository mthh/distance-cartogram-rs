use crate::errors::Error;
use geo_types::Coord;
use pcoa::{apply_pcoa, nalgebra::DMatrix};

/// Takes a duration matrix and returns the coordinates of the points
/// obtained from the PCoA analysis.
///
/// Note that the points are returned in the order of the input durations
/// and are centered around (0, 0). The caller is responsible for
/// translating/scaling/rotating the points as needed to fit them to
/// the reference points (see [`adjustment`] or [`procrustes`] modules for this).
pub fn generate_positions_from_durations(durations: Vec<Vec<f64>>) -> Result<Vec<Coord>, Error> {
    let m = durations.len();
    for item in durations.iter() {
        if item.len() != m {
            return Err(Error::DurationMatrixNotSquare);
        }
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
    Ok(xs
        .iter()
        .zip(ys.iter())
        .map(|(x, y)| Coord { x: *x, y: *y })
        .collect::<Vec<_>>())
}
