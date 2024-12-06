use crate::node::NodeSet;
use crate::point::Point;
use crate::rectangle::Rectangle2D;

/// The grid for interpolating shifting in x and y.
///  Based on Waldo Tobler bidimensional regression.
pub struct Grid<'a> {
    points: &'a [Point],
    nodes: NodeSet,
}

impl<'a> Grid<'a> {
    /// Create a new grid which covers the source points and with a cell size
    /// deduced from the precision.
    pub fn new(points: &'a [Point], precision: f64) -> Grid {
        let mut nodes = NodeSet::new(points, precision);

        for p in points {
            nodes.set_weight_adjacent_nodes(p, 1.0);
        }

        Grid { points, nodes }
    }

    /// Interpolate on the grid the local transformations between the source points and images_points.
    pub fn interpolate(&mut self, image_points: &[Point], n_iter: usize) -> Vec<Point> {
        let rect = Rectangle2D::from_points(self.points);
        let rect_adj = Rectangle2D::from_points(image_points);

        let scale_x = rect.width() / rect_adj.width();
        let scale_y = rect.height() / rect_adj.height();

        let resolution = self.nodes.resolution;
        let width = self.nodes.width;
        let height = self.nodes.height;
        let rect_dim = width * height;

        for k in 0..n_iter {
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
                    self.nodes.update_adjacent_node(src_pt, i, |node| {
                        node.interp.x += adj_x;
                    });
                    let adj_y =
                        u * v * ((dy - qy[i] + sqy) * w[i] + delta_zy[i] * (w[i] * w[i] - sw))
                            / adj_nodes[i].weight;
                    self.nodes.update_adjacent_node(src_pt, i, |node| {
                        node.interp.y += adj_y;
                    });
                }
            }

            let mut p_tmp = Point::new(0.0, 0.0);
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
                            delta = delta.max(p_tmp.distance(&node.interp) / rect_dim as f64);
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
    pub fn get_interp_point(&self, src_point: &Point) -> Point {
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

        Point::new(hx, hy)
    }
}
