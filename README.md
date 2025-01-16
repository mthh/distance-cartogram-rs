# Distance-cartogram-rs

A Rust implementation of distance cartogram (based on Waldo Tobler and Caulette Cauvin work) to be used
to create a fast distance cartogram package for R.


> **Notes**:
> - This is a port of the **[Darcy](https://thema.univ-fcomte.fr/productions/software/darcy/)** standalone software regarding the bidimensional regression and the backgrounds layers deformation.  
All credit for the contribution of the method goes to **Colette Cauvin** *(Théma - Univ. Franche-Comté)* and for the reference Java implementation goes to **Gilles Vuidel** *(Théma - Univ. Franche-Comté)*.
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
the images points from the durations between all the source points (this is a multipolar displacement - there is no reference point, all the points might be moved - used for multipolar distance cartograms).
Internally this function performs [Principal Coordinates Analysis (PCoA)](https://en.wikipedia.org/wiki/Multidimensional_scaling#Classical_multidimensional_scaling) on the durations matrix to get the relative positions of the points. We say "relative positions" because the points returned are still centered on (0, 0) and can't be used directly to create a distance cartogram. 

You then need to fit these points to the source points (using either the `adjustment::adjust` or the `procrustes::procrustes` function) to get the final image points that can be used with the `Grid` struct to create distance cartograms.

See the examples in the [`examples`](./examples) directory for more details:

- from two sets of points: [`from-2-point-layers`](./examples/from-2-point-layers.rs) (`cargo run --example from-2-point-layers --release`), demonstrating the `Grid` core feature.
- from a reference point and durations: [`from-reference-point-and-durations`](./examples/from-reference-point-and-durations.rs) (`cargo run --example from-reference-point-and-durations --features moving-points-unipolar --release`), demonstrating the `move_points` function then the `Grid` core feature.
- from points and durations matrix: [`from-points-and-durations-matrix`](./examples/from-points-and-durations-matrix.rs) (`cargo run --example from-points-and-durations-matrix --features moving-points-multipolar --release`), demonstrating the `generate_positions_from_durations` function then the `Grid` core feature.

_**From R**_:

The [distanamo R package](https://github.com/riatelab/distanamo) wrapping this library is currently under development.

### License

**GPL-3.0**
