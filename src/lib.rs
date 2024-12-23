//! This library provides a way to create a distance cartogram from a set of points.
//! For this purpose, this crate applies a spatial comparison method called “bidimensional regression”,
//! developed by Waldo Tobler.
//! This method compares two surfaces described by homologous points whose positions correspond to
//! the phenomenon being studied (such as positions in access time).
//!
//! For this purpose, the main feature exposed is the `Grid` struct, which is the main entry point
//! to apply bidimensional regression to a set of homologous points (called *source points* and *image points*).
//! This can then be used to interpolate any point within the grid (such as the background layers of a map)
//! to create a distance cartogram.
//!
//! This crate also provides a way to move points from a reference point and a set of durations (using
//! the `moving-points` feature). This can be useful if you only have source points and want to
//! create image points from them.
mod bbox;
mod errors;
mod grid;

#[cfg(feature = "moving-points")]
mod moving_points;
mod node;
mod rectangle;

/// Useful utilities for working with the interpolation grid.
pub mod utils;

pub use bbox::BBox;
pub use grid::{Grid, GridType};

#[cfg(feature = "moving-points")]
pub use moving_points::move_points;
#[cfg(feature = "moving-points")]
pub use moving_points::CentralTendency;
