use crate::bbox::BBox;
use crate::errors::Error;
use crate::node::NodeSet;
use crate::rectangle::Rectangle2D;
use crate::utils;
use crate::utils::distance_sq;
use geo_types::Coord;
#[cfg(feature = "parallel")]
use rayon::prelude::*;
use std::fmt::Debug;

/// The type of grid to retrieve (source or interpolated,
/// see [`Grid::get_grid`](Grid::get_grid) method).
#[derive(Eq, PartialEq, Ord, PartialOrd, Debug)]
pub enum GridType {
    Source,
    Interpolated,
}

/// The Root Mean Squared Error (RMSE) between two sets of points
/// (the total RMSE and the RMSE for the x and y directions).
#[derive(Debug, Clone, Copy)]
pub struct RMSE {
    pub rmse: f64,
    pub rmse_x: f64,
    pub rmse_y: f64,
}

/// The grid for interpolating and deforming geometries.
/// Based on Waldo Tobler bidimensional regression.
///
/// Waldo Tobler's bidimensional regression is a statistical method
/// used to compare two sets of spatial data by analyzing the relationship
/// between corresponding points. It is particularly useful in geography
/// for comparing different maps or spatial representations to understand
/// how one dataset can be transformed to approximate another.
pub struct Grid {
    nodes: NodeSet,
    interpolated_points: Vec<Coord>,
    mae: f64,
    r_squared: f64,
    rmse_interpolated_image: RMSE,
    rmse_interpolated_source: RMSE,
}

impl Grid {
    /// Create a new grid which covers the source points and with a cell size
    /// deduced from the precision.
    /// During its creation, the nodes of the grid will be adjusted
    /// to minimize the differences between the source and the image points
    /// using Waldo Tobler's bidimensional regression.
    ///
    /// This then allows to interpolate any point on the grid
    /// (enabling the deformation of geometries such as background layers)
    /// and to retrieve useful metrics about the regression (MAE, RMSE,
    /// R-squared and deformation strength).
    ///
    /// If the bbox is not provided, the grid dimension will be deduced from
    /// the source points.
    /// If the bbox provided does not cover all the source points, the grid will
    /// be extended to cover all the source points.
    ///
    /// The precision controls the size of the grid cells (higher is more precise,
    /// for example 0.5 generally gives a coarse result, 2 a satisfactory result
    /// and 4 a particularly fine result). A precision of 2 is usually a good
    /// default value.
    /// Internally, the precision is the α value used to compute the resolution
    /// of the grid : `resolution = sqrt((dx * dy) / n) * α` (where dx and dy
    /// are the width and height of the bounding box, and n is the number of
    /// source points).
    ///
    /// The number of iterations controls the number of iterations for the
    /// interpolation. It is generally 4 times the square root of the number of
    /// points (see [`get_nb_iterations`](crate::utils::get_nb_iterations)
    /// helper function for computing it from the number of points).
    ///
    /// Note that the number of source points must be equal to the number of
    /// image points, and they must be given in the same order (as they are
    /// homologous points).
    pub fn new(
        source_points: &[Coord],
        image_points: &[Coord],
        precision: f64,
        n_iter: usize,
        bbox: Option<BBox>,
    ) -> Result<Grid, Error> {
        if (source_points.len() != image_points.len()) || source_points.is_empty() {
            return Err(Error::InvalidInputPointsLength);
        }
        let mut nodes = NodeSet::new(source_points, precision, bbox);

        for p in source_points {
            nodes.set_weight_adjacent_nodes(p, 1.0);
        }

        let mut g = Grid {
            nodes,
            interpolated_points: vec![],
            mae: 0.0,
            rmse_interpolated_image: RMSE {
                rmse: 0.0,
                rmse_x: 0.0,
                rmse_y: 0.0,
            },
            rmse_interpolated_source: RMSE {
                rmse: 0.0,
                rmse_x: 0.0,
                rmse_y: 0.0,
            },
            r_squared: 0.0,
        };
        g.interpolate(source_points, image_points, n_iter);
        Ok(g)
    }

    /// Interpolate on the grid the local transformations between
    /// the source points and images_points.
    /// This method performs bidimensional regression by iteratively
    /// adjusting a grid of nodes to minimize the differences between
    /// the source and image points.
    fn interpolate(&mut self, points: &[Coord], image_points: &[Coord], n_iter: usize) {
        // let rect = Rectangle2D::from_points(self.points);
        // let rect_adj = Rectangle2D::from_points(image_points);
        let mut rect = Rectangle2D::new(0., 0., -1., -1.);
        let mut rect_adj = Rectangle2D::new(0., 0., -1., -1.);

        for pt in points {
            rect.add(pt);
        }
        for pt in image_points {
            rect_adj.add(pt);
        }

        let scale_x = rect_adj.width() / rect.width();
        let scale_y = rect_adj.height() / rect.height();

        let resolution = self.nodes.resolution;
        let width = self.nodes.width;
        let height = self.nodes.height;
        let rect_dim = width * height;

        for _k in 0..n_iter {
            for (src_pt, adj_pt) in points.iter().zip(image_points) {
                let adj_nodes = self.nodes.get_adjacent_nodes(src_pt);
                let smoothed_nodes = [
                    self.nodes
                        .get_smoothed(adj_nodes[0].i, adj_nodes[0].j, scale_x, scale_y),
                    self.nodes
                        .get_smoothed(adj_nodes[1].i, adj_nodes[1].j, scale_x, scale_y),
                    self.nodes
                        .get_smoothed(adj_nodes[2].i, adj_nodes[2].j, scale_x, scale_y),
                    self.nodes
                        .get_smoothed(adj_nodes[3].i, adj_nodes[3].j, scale_x, scale_y),
                ];

                let ux1 = src_pt.x - adj_nodes[0].source.x;
                let ux2 = resolution - ux1;
                let vy1 = src_pt.y - adj_nodes[2].source.y;
                let vy2 = resolution - vy1;
                let u = 1. / (ux1 * ux1 + ux2 * ux2);
                let v = 1. / (vy1 * vy1 + vy2 * vy2);
                let w = [vy1 * ux2, vy1 * ux1, vy2 * ux2, vy2 * ux1];
                let mut qx = [0., 0., 0., 0.];
                let mut qy = [0., 0., 0., 0.];
                let mut delta_zx = [0., 0., 0., 0.];
                let mut delta_zy = [0., 0., 0., 0.];
                let (mut sqx, mut sqy, mut sw) = (0., 0., 0.);
                for i in 0..4 {
                    sw += w[i].powi(2);
                    delta_zx[i] = adj_nodes[i].interp.x - smoothed_nodes[i].x;
                    delta_zy[i] = adj_nodes[i].interp.y - smoothed_nodes[i].y;
                    qx[i] = w[i] * delta_zx[i];
                    qy[i] = w[i] * delta_zy[i];
                    sqx += qx[i];
                    sqy += qy[i];
                }

                // Compute the local transformation using bilinear interpolation
                let hx1 = ux1 / resolution * (adj_nodes[1].interp.x - adj_nodes[0].interp.x)
                    + adj_nodes[0].interp.x;
                let hx2 = ux1 / resolution * (adj_nodes[3].interp.x - adj_nodes[2].interp.x)
                    + adj_nodes[2].interp.x;
                let hx = vy1 / resolution * (hx1 - hx2) + hx2;
                let hy1 = ux1 / resolution * (adj_nodes[1].interp.y - adj_nodes[0].interp.y)
                    + adj_nodes[0].interp.y;
                let hy2 = ux1 / resolution * (adj_nodes[3].interp.y - adj_nodes[2].interp.y)
                    + adj_nodes[2].interp.y;
                let hy = vy1 / resolution * (hy1 - hy2) + hy2;

                let delta_x = adj_pt.x - hx;
                let delta_y = adj_pt.y - hy;
                let dx = delta_x * resolution * resolution;
                let dy = delta_y * resolution * resolution;

                for i in 0..4 {
                    let adj_x =
                        u * v * ((dx - qx[i] + sqx) * w[i] + delta_zx[i] * (w[i] * w[i] - sw))
                            / adj_nodes[i].weight;
                    let adj_y =
                        u * v * ((dy - qy[i] + sqy) * w[i] + delta_zy[i] * (w[i] * w[i] - sw))
                            / adj_nodes[i].weight;
                    self.nodes.update_adjacent_node(src_pt, i, |node| {
                        node.interp.x += adj_x;
                        node.interp.y += adj_y;
                    });
                }
            }

            // Smooth the grid by updating the nodes interpolated
            // position and check for convergence
            let mut p_tmp = Coord { x: 0., y: 0. };
            for l in 0..(width * height) {
                let mut delta = 0.0f64;
                for i in 0..height {
                    for j in 0..width {
                        if self.nodes.get_node(i, j).weight == 0. {
                            let p = self.nodes.get_smoothed(i, j, scale_x, scale_y);
                            let node = self.nodes.get_mut_node(i, j);
                            p_tmp.x = node.interp.x;
                            p_tmp.y = node.interp.y;
                            node.interp.x = p.x;
                            node.interp.y = p.y;
                            delta = delta.max(distance_sq(&p_tmp, &node.interp) / rect_dim as f64);
                        }
                    }
                }
                if l > 5 && delta.sqrt() < 0.0001 {
                    break;
                }
            }
        }

        self.interpolated_points = points.iter().map(|p| self._get_interp_point(p)).collect();
        self.mae = utils::mae(image_points, &self.interpolated_points);
        self.r_squared = utils::r_squared(image_points, &self.interpolated_points);
        self.rmse_interpolated_image = utils::rmse(&self.interpolated_points, image_points);
        self.rmse_interpolated_source = utils::rmse(points, &self.interpolated_points);
    }

    /// Interpolate the point src_point on the transformed grid.
    /// This is useful for deforming geometries and the logic of this function is
    /// used internally by the [`interpolate_layer`](Grid::interpolate_layer) method.
    pub fn get_interp_point(&self, src_point: &Coord) -> Result<Coord, Error> {
        if !self.bbox().contains(src_point) {
            return Err(Error::PointNotInBBox);
        }
        Ok(self._get_interp_point(src_point))
    }

    fn _get_interp_point(&self, src_point: &Coord) -> Coord {
        let adj_nodes = self.nodes.get_adjacent_nodes_ref(src_point);
        let resolution = self.nodes.resolution;
        let ux1 = src_point.x - adj_nodes[0].source.x;
        let vy1 = src_point.y - adj_nodes[2].source.y;
        let hx1 = ux1 / resolution * (adj_nodes[1].interp.x - adj_nodes[0].interp.x)
            + adj_nodes[0].interp.x;
        let hx2 = ux1 / resolution * (adj_nodes[3].interp.x - adj_nodes[2].interp.x)
            + adj_nodes[2].interp.x;
        let hx = vy1 / resolution * (hx1 - hx2) + hx2;
        let hy1 = ux1 / resolution * (adj_nodes[1].interp.y - adj_nodes[0].interp.y)
            + adj_nodes[0].interp.y;
        let hy2 = ux1 / resolution * (adj_nodes[3].interp.y - adj_nodes[2].interp.y)
            + adj_nodes[2].interp.y;
        let hy = vy1 / resolution * (hy1 - hy2) + hy2;

        Coord { x: hx, y: hy }
    }

    /// Returns the geometry of the grid (either source grid or interpolated grid).
    /// The grid is returned as a collection of geo_types polygons.
    pub fn get_grid(&self, grid_type: GridType) -> Vec<geo_types::Polygon> {
        let mut result = Vec::with_capacity((self.nodes.height - 1) * (self.nodes.width - 1));
        let point_getter = match grid_type {
            GridType::Source => |node: &crate::node::Node| node.source,
            GridType::Interpolated => |node: &crate::node::Node| node.interp,
        };
        for i in 0..(self.nodes.height - 1) {
            for j in 0..(self.nodes.width - 1) {
                result.push(geo_types::Polygon::new(
                    // Geo-types is closing the polygon,
                    // so we don't need to repeat the first point ourselves
                    vec![
                        point_getter(self.nodes.get_node(i, j)),
                        point_getter(self.nodes.get_node(i + 1, j)),
                        point_getter(self.nodes.get_node(i + 1, j + 1)),
                        point_getter(self.nodes.get_node(i, j + 1)),
                    ]
                    .into(),
                    vec![],
                ));
            }
        }

        result
    }

    fn get_diff(&self, i: usize, j: usize) -> [f64; 4] {
        let mut diff = [0.; 4];
        let i = i as isize;
        let j = j as isize;
        let n = if self.nodes.is_in_grid(i, j) {
            Some(self.nodes.get_node(i as usize, j as usize))
        } else {
            None
        };
        let ny1 = if self.nodes.is_in_grid(i - 1, j) {
            Some(self.nodes.get_node((i - 1) as usize, j as usize))
        } else {
            None
        };
        let ny2 = if self.nodes.is_in_grid(i + 1, j) {
            Some(self.nodes.get_node((i + 1) as usize, j as usize))
        } else {
            None
        };
        let nx1 = if self.nodes.is_in_grid(i, j - 1) {
            Some(self.nodes.get_node(i as usize, (j - 1) as usize))
        } else {
            None
        };
        let nx2 = if self.nodes.is_in_grid(i, j + 1) {
            Some(self.nodes.get_node(i as usize, (j + 1) as usize))
        } else {
            None
        };
        if nx1.is_none() {
            diff[0] = (nx2.unwrap().interp.x - n.unwrap().interp.x) / self.nodes.resolution;
            diff[1] = (nx2.unwrap().interp.y - n.unwrap().interp.y) / self.nodes.resolution;
        } else if nx2.is_none() {
            diff[0] = (n.unwrap().interp.x - nx1.unwrap().interp.x) / self.nodes.resolution;
            diff[1] = (n.unwrap().interp.y - nx1.unwrap().interp.y) / self.nodes.resolution;
        } else {
            diff[0] =
                (nx2.unwrap().interp.x - nx1.unwrap().interp.x) / (2. * self.nodes.resolution);
            diff[1] =
                (nx2.unwrap().interp.y - nx1.unwrap().interp.y) / (2. * self.nodes.resolution);
        }

        if ny1.is_none() {
            diff[2] = (n.unwrap().interp.x - ny2.unwrap().interp.x) / self.nodes.resolution;
            diff[3] = (n.unwrap().interp.y - ny2.unwrap().interp.y) / self.nodes.resolution;
        } else if ny2.is_none() {
            diff[2] = (ny1.unwrap().interp.x - n.unwrap().interp.x) / self.nodes.resolution;
            diff[3] = (ny1.unwrap().interp.y - n.unwrap().interp.y) / self.nodes.resolution;
        } else {
            diff[2] =
                (ny1.unwrap().interp.x - ny2.unwrap().interp.x) / (2. * self.nodes.resolution);
            diff[3] =
                (ny1.unwrap().interp.y - ny2.unwrap().interp.y) / (2. * self.nodes.resolution);
        }
        diff
    }

    /// Compute the deformation strength for the node at position (i, j)
    pub fn node_deformation_strength(&self, i: usize, j: usize) -> f64 {
        let diff = self.get_diff(i, j);
        ((diff[0].powi(2) + diff[1].powi(2) + diff[2].powi(2) + diff[3].powi(2)) / 2.).sqrt()
    }

    /// Compute the average deformation strength for the grid
    pub fn deformation_strength(&self) -> f64 {
        (self.sum_squared_deformation_strength() / (self.nodes.width * self.nodes.height) as f64)
            .sqrt()
    }

    /// Retrieve the resolution value
    /// (computed from the precision given at the grid creation)
    pub fn resolution(&self) -> f64 {
        self.nodes.resolution
    }

    /// Compute the sum of squared deformation strength for the grid
    pub fn sum_squared_deformation_strength(&self) -> f64 {
        let mut m2 = 0.;
        for i in 0..self.nodes.height {
            for j in 0..self.nodes.width {
                let diff = self.get_diff(i, j);
                m2 += (diff[0].powi(2) + diff[1].powi(2) + diff[2].powi(2) + diff[3].powi(2)) / 2.;
            }
        }
        m2
    }

    /// Retrieve the bbox of the grid
    pub fn bbox(&self) -> BBox {
        self.nodes.zone.as_bbox()
    }

    #[cfg(feature = "parallel")]
    /// Interpolate a collection of geo_types geometries on the interpolation grid
    /// in parallel using rayon.
    ///
    /// This is useful for interpolating the various geometries of a layer in parallel
    /// while [`interpolate_layers_par`](Grid::interpolate_layers_par) is useful for
    /// interpolating multiple layers at once.
    pub fn interpolate_layer_par(
        &self,
        geometries: &[geo_types::Geometry],
    ) -> Result<Vec<geo_types::Geometry>, Error> {
        let bbox = BBox::from_geometries(geometries);
        if !self.bbox().contains_bbox(&bbox) {
            return Err(Error::GeometriesNotInBBox);
        }
        let result: Vec<geo_types::Geometry> = geometries
            .par_iter()
            .map(|geom| self.interpolate_geom(geom))
            .collect();
        Ok(result)
    }

    #[cfg(feature = "parallel")]
    /// Interpolate a collection of "layers" (collection of geo_types geometries) on the interpolation grid
    /// in parallel using rayon.
    ///
    /// This is useful for interpolating multiple layers at once while [`interpolate_layer_par`](Grid::interpolate_layer_par)
    /// is useful for interpolating a single layer in parallel.
    pub fn interpolate_layers_par(
        &self,
        layers: &[Vec<geo_types::Geometry>],
    ) -> Result<Vec<Vec<geo_types::Geometry>>, Error> {
        layers
            .par_iter()
            .map(|geometries| self.interpolate_layer(geometries))
            .collect::<Result<Vec<Vec<geo_types::Geometry>>, Error>>()
    }

    fn interpolate_geom(&self, geom: &geo_types::Geometry) -> geo_types::Geometry {
        match geom {
            geo_types::Geometry::Point(p) => {
                geo_types::Geometry::Point(geo_types::Point(self._get_interp_point(&p.0)))
            }
            geo_types::Geometry::MultiPoint(mp) => {
                let mut multi_point: Vec<geo_types::Point> = Vec::with_capacity(mp.len());
                for p in mp.iter() {
                    multi_point.push(self._get_interp_point(&p.0).into());
                }
                geo_types::Geometry::MultiPoint(geo_types::MultiPoint(multi_point))
            }
            geo_types::Geometry::LineString(ls) => {
                let mut line = Vec::with_capacity(ls.0.len());
                for p in ls.0.iter() {
                    line.push(self._get_interp_point(p));
                }
                geo_types::Geometry::LineString(geo_types::LineString(line))
            }
            geo_types::Geometry::MultiLineString(mls) => {
                let mut multi_line = Vec::with_capacity(mls.0.len());
                for ls in mls.iter() {
                    let mut line = Vec::with_capacity(ls.0.len());
                    for p in ls.0.iter() {
                        line.push(self._get_interp_point(p));
                    }
                    multi_line.push(geo_types::LineString(line));
                }
                geo_types::Geometry::MultiLineString(geo_types::MultiLineString(multi_line))
            }
            geo_types::Geometry::Polygon(poly) => {
                let mut exterior = Vec::with_capacity(poly.exterior().0.len());
                for p in poly.exterior().0.iter() {
                    exterior.push(self._get_interp_point(p));
                }
                let mut interiors = Vec::with_capacity(poly.interiors().len());
                for interior in poly.interiors() {
                    let mut interior_points = Vec::with_capacity(interior.0.len());
                    for p in interior.0.iter() {
                        interior_points.push(self._get_interp_point(p));
                    }
                    interiors.push(interior_points.into());
                }
                geo_types::Geometry::Polygon(geo_types::Polygon::new(exterior.into(), interiors))
            }
            geo_types::Geometry::MultiPolygon(mpoly) => {
                let mut multi_polygon = Vec::with_capacity(mpoly.0.len());
                for poly in mpoly.iter() {
                    let mut exterior = Vec::with_capacity(poly.exterior().0.len());
                    for p in poly.exterior().0.iter() {
                        exterior.push(self._get_interp_point(p));
                    }
                    let mut interiors = Vec::with_capacity(poly.interiors().len());
                    for interior in poly.interiors() {
                        let mut interior_points = Vec::with_capacity(interior.0.len());
                        for p in interior.0.iter() {
                            interior_points.push(self._get_interp_point(p));
                        }
                        interiors.push(interior_points.into());
                    }
                    multi_polygon.push(geo_types::Polygon::new(exterior.into(), interiors));
                }
                geo_types::Geometry::MultiPolygon(geo_types::MultiPolygon(multi_polygon))
            }
            geo_types::Geometry::GeometryCollection(geometries) => {
                geo_types::Geometry::GeometryCollection(
                    geometries
                        .iter()
                        .map(|g| self.interpolate_geom(g))
                        .collect(),
                )
            }
            geo_types::Geometry::Line(l) => {
                let p1 = self._get_interp_point(&l.start);
                let p2 = self._get_interp_point(&l.end);
                geo_types::Geometry::Line(geo_types::Line { start: p1, end: p2 })
            }
            geo_types::Geometry::Triangle(tri) => {
                let v1 = self._get_interp_point(&tri.0);
                let v2 = self._get_interp_point(&tri.1);
                let v3 = self._get_interp_point(&tri.2);
                geo_types::Geometry::Triangle(geo_types::Triangle(v1, v2, v3))
            }
            geo_types::Geometry::Rect(r) => {
                let min = self._get_interp_point(&r.min());
                let max = self._get_interp_point(&r.max());
                geo_types::Geometry::Rect(geo_types::Rect::new(min, max))
            }
        }
    }

    /// Interpolate a collection of geo_types geometries on the interpolation grid.
    pub fn interpolate_layer(
        &self,
        geometries: &[geo_types::Geometry],
    ) -> Result<Vec<geo_types::Geometry>, Error> {
        let bbox = BBox::from_geometries(geometries);
        if !self.bbox().contains_bbox(&bbox) {
            return Err(Error::GeometriesNotInBBox);
        }

        let result = geometries
            .iter()
            .map(|geom| self.interpolate_geom(geom))
            .collect();

        Ok(result)
    }

    /// Retrieve the interpolated points (can be useful for debugging
    /// or computing metrics other than the default ones).
    pub fn interpolated_points(&self) -> &[Coord] {
        &self.interpolated_points
    }

    /// Retrieve the Mean Absolute Error (MAE) between the image points
    /// and the interpolated points.
    /// It measures the average magnitude of the errors in a set of predictions,
    /// without considering their direction.
    pub fn mae(&self) -> f64 {
        self.mae
    }

    /// Retrieve the Root Mean Squared Error (RMSE) between the interpolated points
    /// and the image points.
    /// It measures differences between predicted values and observed values
    /// and gives an idea of the overall accuracy of the regression.
    pub fn rmse_interp_image(&self) -> RMSE {
        self.rmse_interpolated_image
    }

    /// Retrieve the Root Mean Squared Error (RMSE) between the interpolated points
    /// and the source points.
    pub fn rmse_interp_source(&self) -> RMSE {
        self.rmse_interpolated_source
    }

    /// Retrieve the R-squared value between the image points
    /// and the interpolated points.
    /// It measures the proportion of the variance in the dependent variable
    /// that is predictable from the independent variable(s).
    /// It provides an indication of the goodness of fit of the points to the grid.
    /// The R-squared value is between 0 and 1, where 1 indicates a perfect fit.
    pub fn r_squared(&self) -> f64 {
        self.r_squared
    }

    /// Get the dimensions of the grid as (width, height)
    pub fn grid_dimensions(&self) -> (usize, usize) {
        (self.nodes.width, self.nodes.height)
    }
}

impl Debug for Grid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Grid")
            .field("nodes", &self.nodes)
            .field("MAE", &self.mae)
            .field("RMSE (interpolated - image)", &self.rmse_interpolated_image)
            .field(
                "RMSE (interpolated - source)",
                &self.rmse_interpolated_source,
            )
            .field("R²", &self.r_squared)
            .finish()
    }
}
