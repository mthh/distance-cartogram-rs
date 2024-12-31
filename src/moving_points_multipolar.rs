use crate::errors::Error;
use crate::procrustes;
use geo_types::Coord;
use pcoa::{apply_pcoa, nalgebra::DMatrix};

/// Positioning result, containing the coordinates of the points, as well as the various
/// metrics computed during the Procrustes analysis (angle, centroid, error, reflection, scale
/// and translation).
pub struct PositioningResult {
    /// The coordinates of the points after transformation.
    pub points: Vec<Coord>,
    /// The rotation angle.
    pub angle: f64,
    /// The centroid of the points.
    pub centroid: Coord,
    /// The error of the transformation.
    pub error: f64,
    /// Whether the transformation includes a reflection.
    pub reflection: bool,
    /// The scale of the transformation.
    pub scale: f64,
    /// The translation of the transformation.
    pub translation: Coord,
}

impl From<procrustes::ProcrustesResult> for PositioningResult {
    fn from(res: procrustes::ProcrustesResult) -> Self {
        PositioningResult {
            points: res.points,
            angle: res.angle,
            centroid: res.centroid,
            error: res.error,
            reflection: res.reflection,
            scale: res.scale,
            translation: res.translation,
        }
    }
}

impl std::fmt::Debug for PositioningResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PositioningResult")
            .field("angle", &self.angle)
            .field("centroid", &self.centroid)
            .field("error", &self.error)
            .field("reflection", &self.reflection)
            .field("scale", &self.scale)
            .field("translation", &self.translation)
            .finish()
    }
}

/// Takes a duration matrix and a set of reference points and returns the coordinates of the points
/// derived from the PCoA analysis of the duration matrix, rotated, scaled and translated to best
/// fit the reference points using the Procrustes analysis.
pub fn generate_positions_from_durations(
    durations: Vec<Vec<f64>>,
    reference_points: &[Coord<f64>],
) -> Result<PositioningResult, Error> {
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
    Ok(PositioningResult::from(proc_res))
}
