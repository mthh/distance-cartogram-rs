use crate::bbox::BBox;
use crate::rectangle::Rectangle2D;
use geo_types::Coord;

/// A node in the grid.
#[derive(Debug, Clone)]
pub(crate) struct Node {
    /// Position on the grid (line)
    pub i: usize,
    /// Position on the grid (column)
    pub j: usize,
    pub source: Coord,
    pub interp: Coord,
    pub weight: f64,
}

impl Node {
    pub fn new(i: usize, j: usize, source: Coord) -> Node {
        Node {
            i,
            j,
            source,
            interp: source,
            weight: 0.0,
        }
    }
}

/// The internal representation of the grid.
pub(crate) struct NodeSet {
    /// The nodes of the grid
    pub nodes: Vec<Node>,
    /// Envelope of the grid
    pub zone: Rectangle2D,
    /// Size of the cell's grid
    pub resolution: f64,
    /// Number of nodes in X
    pub width: usize,
    /// Number of nodes in Y
    pub height: usize,
}

impl NodeSet {
    pub fn new(points: &[Coord], precision: f64, bbox: Option<BBox>) -> NodeSet {
        let mut zone = if bbox.is_none() {
            // Compute the rectangle from the given points
            Rectangle2D::from_points(points)
        } else {
            // Use the given bounding box to create the rectangle
            let mut r = Rectangle2D::from_bbox(&bbox.unwrap());
            // And extend it to include all source points if necessary
            for p in points {
                r.add(p);
            }
            r
        };
        let resolution =
            1. / precision * (zone.width() * zone.height() / points.len() as f64).sqrt();

        let mut width = (zone.width() / resolution).ceil() as usize + 1;
        let mut height = (zone.height() / resolution).ceil() as usize + 1;

        let dx = width as f64 * resolution - zone.width();
        let dy = height as f64 * resolution - zone.height();

        zone.set_rect_from_center(
            &Coord {
                x: zone.center_x(),
                y: zone.center_y(),
            },
            &Coord {
                x: zone.min_x() - dx / 2.,
                y: zone.min_y() - dy / 2.,
            },
        );

        width += 1;
        height += 1;

        let mut nodes = Vec::with_capacity(width * height);
        let min_x = zone.min_x();
        let max_y = zone.max_y();

        for i in 0..height {
            for j in 0..width {
                nodes.push(Node::new(
                    i,
                    j,
                    Coord {
                        x: min_x + j as f64 * resolution,
                        y: max_y - i as f64 * resolution,
                    },
                ));
            }
        }

        NodeSet {
            nodes,
            zone,
            resolution,
            width,
            height,
        }
    }

    pub fn is_in_grid(&self, i: isize, j: isize) -> bool {
        i < self.height as isize && j < self.width as isize && i >= 0 && j >= 0
    }

    pub fn get_node(&self, i: usize, j: usize) -> &Node {
        &self.nodes[i * self.width + j]
    }

    pub fn get_mut_node(&mut self, i: usize, j: usize) -> &mut Node {
        &mut self.nodes[i * self.width + j]
    }

    fn get_i(&self, p: &Coord) -> usize {
        ((self.zone.max_y() - p.y) / self.resolution).floor() as usize
    }

    fn get_j(&self, p: &Coord) -> usize {
        ((p.x - self.zone.min_x()) / self.resolution).floor() as usize
    }

    pub fn get_adjacent_nodes(&self, point: &Coord) -> [Node; 4] {
        let i = self.get_i(point);
        let j = self.get_j(point);
        [
            self.get_node(i, j).clone(),
            self.get_node(i, j + 1).clone(),
            self.get_node(i + 1, j).clone(),
            self.get_node(i + 1, j + 1).clone(),
        ]
    }

    pub fn update_adjacent_node<F>(&mut self, point: &Coord, i: usize, mut f: F)
    where
        F: FnMut(&mut Node),
    {
        let (i, j) = if i == 0 {
            (self.get_i(point), self.get_j(point))
        } else if i == 1 {
            (self.get_i(point), self.get_j(point) + 1)
        } else if i == 2 {
            (self.get_i(point) + 1, self.get_j(point))
        } else {
            (self.get_i(point) + 1, self.get_j(point) + 1)
        };
        let node = self.get_mut_node(i, j);
        f(node);
    }

    pub fn set_weight_adjacent_nodes(&mut self, point: &Coord, value: f64) {
        let i = self.get_i(point);
        let j = self.get_j(point);
        let n1 = self.get_mut_node(i, j);
        n1.weight += value;
        let n2 = self.get_mut_node(i, j + 1);
        n2.weight += value;
        let n3 = self.get_mut_node(i + 1, j);
        n3.weight += value;
        let n4 = self.get_mut_node(i + 1, j + 1);
        n4.weight += value;
    }

    pub fn get_smoothed(&self, i: usize, j: usize, scale_x: f64, scale_y: f64) -> Coord {
        if i > 1 && j > 1 && i < self.height - 2 && j < self.width - 2 {
            let pa = self.get_node(i - 1, j).interp;
            let pb = self.get_node(i + 1, j).interp;
            let pc = self.get_node(i, j - 1).interp;
            let pd = self.get_node(i, j + 1).interp;
            let pe = self.get_node(i - 1, j - 1).interp;
            let pf = self.get_node(i + 1, j - 1).interp;
            let pg = self.get_node(i + 1, j + 1).interp;
            let ph = self.get_node(i - 1, j + 1).interp;
            let pi = self.get_node(i - 2, j).interp;
            let pj = self.get_node(i + 2, j).interp;
            let pk = self.get_node(i, j - 2).interp;
            let pl = self.get_node(i, j + 2).interp;
            Coord {
                x: (8. * (pa.x + pb.x + pc.x + pd.x)
                    - 2. * (pe.x + pf.x + pg.x + ph.x)
                    - (pi.x + pj.x + pk.x + pl.x))
                    / 20.,
                y: (8. * (pa.y + pb.y + pc.y + pd.y)
                    - 2. * (pe.y + pf.y + pg.y + ph.y)
                    - (pi.y + pj.y + pk.y + pl.y))
                    / 20.,
            }
        } else {
            let mut nb = 0;
            let mut sx = 0.;
            let mut sy = 0.;
            if i > 0 {
                let n = &self.get_node(i - 1, j).interp;
                sx += n.x;
                sy += n.y;
                nb += 1;
            } else {
                sy += self.resolution * scale_y;
            }
            if j > 0 {
                let n = &self.get_node(i, j - 1).interp;
                sx += n.x;
                sy += n.y;
                nb += 1;
            } else {
                sx -= self.resolution * scale_x;
            }
            if i < self.height - 1 {
                let n = &self.get_node(i + 1, j).interp;
                sx += n.x;
                sy += n.y;
                nb += 1;
            } else {
                sy -= self.resolution * scale_y;
            }
            if j < self.width - 1 {
                let n = &self.get_node(i, j + 1).interp;
                sx += n.x;
                sy += n.y;
                nb += 1;
            } else {
                sx += self.resolution * scale_x;
            }
            Coord {
                x: sx / nb as f64,
                y: sy / nb as f64,
            }
        }
    }
}
