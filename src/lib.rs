mod bbox;
mod errors;
mod grid;

#[cfg(feature = "moving-points")]
mod moving_points;
mod node;
mod rectangle;
mod utils;

pub use bbox::BBox;
pub use grid::{Grid, GridType};
pub use utils::get_nb_iterations;

#[cfg(test)]
mod tests {

    #[test]
    fn it_works() {
        assert_eq!(1, 1);
    }
}
