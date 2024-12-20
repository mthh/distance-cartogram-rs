use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("The number of source points and image points must be equal")]
    InvalidInputPointsLength,
    #[cfg(feature = "moving-points")]
    #[error("The number of source points and duration measurements must be equal")]
    InvalidInputDurationsLength,
    #[cfg(feature = "moving-points")]
    #[error("No reference point found")]
    NoReferencePoint,
    #[error("The provided geometries don't fall inside the bounding box of the grid")]
    GeometriesNotInBBox,
    #[error("The provided point don't fall inside the bounding box of the grid")]
    PointNotInBBox,
}
