use crate::bbox::BBox;
use crate::node::NodeSet;
use crate::rectangle::Rectangle2D;
use crate::utils::distance_sq;
use geo_types::Coord;

#[derive(Eq, PartialEq, Ord, PartialOrd, Debug)]
pub enum GridType {
    Source,
    Interpolated,
}

/// The grid for interpolating shifting in x and y.
///  Based on Waldo Tobler bidimensional regression.
pub struct Grid<'a> {
    points: &'a [Coord],
    nodes: NodeSet,
}

impl<'a> Grid<'a> {
    /// Create a new grid which covers the source points and with a cell size
    /// deduced from the precision.
    pub fn new(points: &'a [Coord], precision: f64, bbox: Option<BBox>) -> Grid {
        let mut nodes = NodeSet::new(points, precision, bbox);

        for p in points {
            nodes.set_weight_adjacent_nodes(p, 1.0);
        }

        Grid { points, nodes }
    }

    /// Interpolate on the grid the local transformations between the source points and images_points.
    pub fn interpolate(&mut self, image_points: &[Coord], n_iter: usize) -> Vec<Coord> {
        // let rect = Rectangle2D::from_points(self.points);
        // let rect_adj = Rectangle2D::from_points(image_points);
        let mut rect = Rectangle2D::new(0., 0., -1., -1.);
        let mut rect_adj = Rectangle2D::new(0., 0., -1., -1.);

        for pt in self.points {
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
            for (src_pt, adj_pt) in self.points.iter().zip(image_points) {
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

            let mut p_tmp = Coord { x: 0., y: 0. };
            for l in 0..(width * height) {
                let mut delta = 0.0f64;
                for i in 0..height {
                    for j in 0..width {
                        let p = self.nodes.get_smoothed(i, j, scale_x, scale_y);
                        let node = self.nodes.get_mut_node(i, j);
                        if node.weight == 0. {
                            p_tmp.x = node.interp.x;
                            p_tmp.y = node.interp.y;
                            // let p = self.nodes.get_smoothed(i, j, scale_x, scale_y);
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

        self.points
            .iter()
            .map(|p| self.get_interp_point(p))
            .collect::<Vec<_>>()
    }

    /// Interpolate the point src_point on the transformed grid
    pub fn get_interp_point(&self, src_point: &Coord) -> Coord {
        let adj_nodes = self.nodes.get_adjacent_nodes(src_point);
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

    /// Returns the coordinates of the grid
    pub fn get_grid(&self, grid_type: GridType) -> Vec<geo_types::Polygon> {
        let mut result = Vec::with_capacity((self.nodes.height - 1) * (self.nodes.width - 1));
        if grid_type == GridType::Source {
            for i in 0..(self.nodes.height - 1) {
                for j in 0..(self.nodes.width - 1) {
                    result.push(geo_types::Polygon::new(
                        vec![
                            self.nodes.get_node(i, j).source.clone(),
                            self.nodes.get_node(i + 1, j).source.clone(),
                            self.nodes.get_node(i + 1, j + 1).source.clone(),
                            self.nodes.get_node(i, j + 1).source.clone(),
                        ]
                        .into(),
                        vec![],
                    ));
                }
            }
        } else {
            for i in 0..(self.nodes.height - 1) {
                for j in 0..(self.nodes.width - 1) {
                    result.push(geo_types::Polygon::new(
                        vec![
                            self.nodes.get_node(i, j).interp.clone(),
                            self.nodes.get_node(i + 1, j).interp.clone(),
                            self.nodes.get_node(i + 1, j + 1).interp.clone(),
                            self.nodes.get_node(i, j + 1).interp.clone(),
                        ]
                        .into(),
                        vec![],
                    ));
                }
            }
        }

        result
    }

    /// Interpolate a collection of geo_types geometries on the interpolation grid.
    pub fn interpolate_layer(
        &self,
        geometries: &[geo_types::Geometry],
    ) -> Vec<geo_types::Geometry> {
        let mut result = Vec::with_capacity(geometries.len());
        for geom in geometries {
            match geom {
                geo_types::Geometry::Point(p) => {
                    result.push(geo_types::Geometry::Point(geo_types::Point(
                        self.get_interp_point(&p.0),
                    )));
                }
                geo_types::Geometry::MultiPoint(mp) => {
                    let mut multi_point: Vec<geo_types::Point> = Vec::with_capacity(mp.len());
                    for p in mp.iter() {
                        multi_point.push(self.get_interp_point(&p.0).into());
                    }
                    result.push(geo_types::Geometry::MultiPoint(geo_types::MultiPoint(
                        multi_point,
                    )));
                }
                geo_types::Geometry::LineString(ls) => {
                    let mut line = Vec::with_capacity(ls.0.len());
                    for p in ls.0.iter() {
                        line.push(self.get_interp_point(&p));
                    }
                    result.push(geo_types::Geometry::LineString(geo_types::LineString(line)));
                }
                geo_types::Geometry::MultiLineString(mls) => {
                    let mut multi_line = Vec::with_capacity(mls.0.len());
                    for ls in mls.iter() {
                        let mut line = Vec::with_capacity(ls.0.len());
                        for p in ls.0.iter() {
                            line.push(self.get_interp_point(&p));
                        }
                        multi_line.push(geo_types::LineString(line));
                    }
                    result.push(geo_types::Geometry::MultiLineString(
                        geo_types::MultiLineString(multi_line),
                    ));
                }
                geo_types::Geometry::Polygon(poly) => {
                    let mut exterior = Vec::with_capacity(poly.exterior().0.len());
                    for p in poly.exterior().0.iter() {
                        exterior.push(self.get_interp_point(&p));
                    }
                    let mut interiors = Vec::with_capacity(poly.interiors().len());
                    for interior in poly.interiors() {
                        let mut interior_points = Vec::with_capacity(interior.0.len());
                        for p in interior.0.iter() {
                            interior_points.push(self.get_interp_point(&p));
                        }
                        interiors.push(interior_points.into());
                    }
                    result.push(geo_types::Geometry::Polygon(geo_types::Polygon::new(
                        exterior.into(),
                        interiors.into(),
                    )));
                }
                geo_types::Geometry::MultiPolygon(mpoly) => {
                    let mut multi_polygon = Vec::with_capacity(mpoly.0.len());
                    for poly in mpoly.iter() {
                        let mut exterior = Vec::with_capacity(poly.exterior().0.len());
                        for p in poly.exterior().0.iter() {
                            exterior.push(self.get_interp_point(&p));
                        }
                        let mut interiors = Vec::with_capacity(poly.interiors().len());
                        for interior in poly.interiors() {
                            let mut interior_points = Vec::with_capacity(interior.0.len());
                            for p in interior.0.iter() {
                                interior_points.push(self.get_interp_point(&p));
                            }
                            interiors.push(interior_points.into());
                        }
                        multi_polygon
                            .push(geo_types::Polygon::new(exterior.into(), interiors.into()));
                    }
                    result.push(geo_types::Geometry::MultiPolygon(geo_types::MultiPolygon(
                        multi_polygon,
                    )));
                }
                geo_types::Geometry::GeometryCollection(geometries) => {
                    result = self.interpolate_layer(&geometries.0);
                }
                geo_types::Geometry::Line(l) => {
                    let p1 = self.get_interp_point(&l.start);
                    let p2 = self.get_interp_point(&l.end);
                    result.push(geo_types::Geometry::Line(geo_types::Line {
                        start: p1,
                        end: p2,
                    }));
                }
                geo_types::Geometry::Triangle(tri) => {
                    let v1 = self.get_interp_point(&tri.0);
                    let v2 = self.get_interp_point(&tri.1);
                    let v3 = self.get_interp_point(&tri.2);
                    result.push(geo_types::Geometry::Triangle(geo_types::Triangle(
                        v1, v2, v3,
                    )));
                }
                geo_types::Geometry::Rect(r) => {
                    let min = self.get_interp_point(&r.min());
                    let max = self.get_interp_point(&r.max());
                    result.push(geo_types::Geometry::Rect(geo_types::Rect::new(min, max)));
                }
            }
        }

        result
    }
}
