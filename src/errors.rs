use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("The number of source points and image points must be equal")]
    InvalidInputPointsLength,
    #[cfg(feature = "moving-points")]
    #[error("The number of source points and time measurements must be equal")]
    InvalidInputTimesLength,
    #[cfg(feature = "moving-points")]
    #[error("No reference point found")]
    NoReferencePoint,
}
