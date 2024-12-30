# Distance-cartogram-rs

A Rust implementation of distance cartogram (based on Waldo Tobler's bidimensional regression) to be used to create a fast distance cartogram package for R.


> **Notes**:
> - This is a port of the **[Darcy](https://thema.univ-fcomte.fr/productions/software/darcy/)** standalone software regarding the bidimensional regression and the backgrounds layers deformation.  
All credits for the contribution of the method goes to **Colette Cauvin** *(Théma - Univ. Franche-Comté)* and for the reference Java implementation goes to **Gilles Vuidel** *(Théma - Univ. Franche-Comté)*.
> - This method is also available as a **QGIS plugin** ([GitHub repository](https://github.com/mthh/QgisDistanceCartogramPlugin) / [QGIS plugin repository](https://plugins.qgis.org/plugins/dist_cartogram/)).

### Usage

_**From Rust**_:

**Core feature**:

The core feature provided by this library is a `Grid` struct that have to be initialized from two sets of homologous
points (called the *source points* and the *image points*, and provided as two `&[geo_types::Coord<f64>]`).

During the initialization, a grid is created and the initial interpolation step is performed.

The grid can then be used to deform a set of points (provided as a `&[geo_types::Geometry]` - all types of geometries
are supported) to their corresponding positions in the cartogram.

**Additional features**:

This crate also provides a `move_points` function (under the `moving-points-unipolar` feature gate) that can be used to create
the images points from the source points and the time between them (this is a unipolar displacement - based on a reference point that is
not moved - used for unipolar distance cartograms).
This function returns the *image points* that can be used with the `Grid` struct to create distance cartograms.

This crate also provides a `generate_positions_from_durations` function (under the `moving-points-multipolar` feature gate) that can be used to create
the images points from the source points and the time between them (this is a multipolar displacement - there is no reference point, all the points might be moved - used for multipolar distance cartograms).
Internally this function performs PCoA on the durations matrix to get the relative positions of the points, then it
uses Ordinary Procrustes Analysis to compare the positions obtained from the durations matrix with the source points 
in order to get the *image points* that can be used with the `Grid` struct to create distance cartograms.

See the examples in the [`examples`](./examples) directory for more details:

- `cargo run --example from-2-point-layers --release`
- `cargo run --example from-points-and-durations --features moving-points --release`
- `cargo run --example from-points-and-duration-matrix --features moving-points-multipolar --release`

_**From R**_:

The [distanamo R package](https://github.com/riatelab/distanamo) wrapping this library is currently under development.

### License

**GPL-3.0**
