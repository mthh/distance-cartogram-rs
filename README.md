# Distance-cartogram-rs

A Rust implementation of distance cartogram (based on Waldo Tobler's bidimensional regression) to be used to create a fast distance cartogram package for R.


> **Notes**:
> - This is a port of the **[Darcy](https://thema.univ-fcomte.fr/productions/software/darcy/)** standalone software regarding the bidimensional regression and the backgrounds layers deformation.  
All credits for the contribution of the method goes to **Colette Cauvin** *(Théma - Univ. Franche-Comté)* and for the reference Java implementation goes to **Gilles Vuidel** *(Théma - Univ. Franche-Comté)*.
> - This method is also available as a **QGIS plugin** ([GitHub repository](https://github.com/mthh/QgisDistanceCartogramPlugin) / [QGIS plugin repository](https://plugins.qgis.org/plugins/dist_cartogram/)).

### Usage

_**From Rust**_:

The core features provided by this library is a `Grid` struct that have to be initialized from two sets of homologous points
(called the *source points* and the *image points*, and provided as two `&[geo_types::Coord<f64>]`).

During the initialization, a grid is created and the initial interpolation step is performed.

The grid can then be used to deform a set of points (provided as a `&[geo_types::Geometry]` - all types of geometries are supported)
to their corresponding positions in the cartogram.

See the examples in the [`examples`](./examples) directory for more details.

_**From R**_:

The R package wrapping this library is currently under development.

### License

**GPL-3.0**
