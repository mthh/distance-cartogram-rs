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

#[cfg(test)]
mod tests {

    #[test]
    fn it_works() {
        assert_eq!(1, 1);
    }
}
