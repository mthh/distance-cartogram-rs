use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("The number of source points and image points must be equal")]
    InvalidInputPointsLength,

    #[error("The provided geometries don't fall inside the bounding box of the grid")]
    GeometriesNotInBBox,

    #[error("The provided point don't fall inside the bounding box of the grid")]
    PointNotInBBox,

    #[error("The two sets of input points for Procrustes analysis must have the same length")]
    ProcrustesInputLengthMismatch,

    #[cfg(feature = "moving-points-unipolar")]
    #[error("The number of source points and duration measurements must be equal")]
    InvalidInputDurationsLength,

    #[cfg(feature = "moving-points-unipolar")]
    #[error("No reference point found")]
    NoReferencePoint,

    #[cfg(feature = "moving-points-multipolar")]
    #[error("The duration matrix is not square")]
    DurationMatrixNotSquare,

    #[cfg(feature = "moving-points-multipolar")]
    #[error("An error occurred during the PCoA analysis")]
    PCoAUnsuccessful,
}
