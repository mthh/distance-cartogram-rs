use crate::point::Point;
use crate::rectangle::Rectangle2D;

/// A node in the grid.
#[derive(Debug, Clone)]
pub(crate) struct Node {
    /// Position on the grid (line)
    pub i: usize,
    /// Position on the grid (column)
    pub j: usize,
    pub source: Point,
    pub interp: Point,
    pub weight: f64,
}

impl Node {
    pub fn new(i: usize, j: usize, source: Point) -> Node {
        Node {
            i,
            j,
            source: source.clone(),
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
    pub fn new(points: &[Point], precision: f64) -> NodeSet {
        let mut zone = Rectangle2D::from_points(points);
        let resolution =
            1. / precision * (zone.width() * zone.height() / points.len() as f64).sqrt();

        let mut width = (zone.width() / resolution).ceil() as usize + 1;
        let mut height = (zone.height() / resolution).ceil() as usize + 1;

        let dx = width as f64 * resolution - zone.width();
        let dy = height as f64 * resolution - zone.height();

        zone.set_rect_from_center(
            &Point::new(zone.center_x(), zone.center_y()),
            &Point::new(zone.min_x() - dx / 2., zone.min_y() - dy / 2.),
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
                    Point::new(min_x + j as f64 * resolution, max_y - i as f64 * resolution),
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

    pub fn get_node(&self, i: usize, j: usize) -> &Node {
        &self.nodes[i * self.width + j]
    }

    pub fn get_mut_node(&mut self, i: usize, j: usize) -> &mut Node {
        &mut self.nodes[i * self.width + j]
    }

    fn get_i(&self, p: &Point) -> usize {
        ((self.zone.max_y() - p.y) / self.resolution).round() as usize
    }

    fn get_j(&self, p: &Point) -> usize {
        ((p.x - self.zone.min_x()) / self.resolution).round() as usize
    }

    pub fn get_adjacent_nodes(&self, point: &Point) -> [Node; 4] {
        let i = self.get_i(point);
        let j = self.get_j(point);
        [
            self.get_node(i, j).clone(),
            self.get_node(i, j + 1).clone(),
            self.get_node(i + 1, j).clone(),
            self.get_node(i + 1, j + 1).clone(),
        ]
    }

    pub fn update_adjacent_node<F>(&mut self, point: &Point, i: usize, mut f: F)
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

    pub fn set_weight_adjacent_nodes(&mut self, point: &Point, value: f64) {
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

    pub fn get_smoothed(&self, i: usize, j: usize, scale_x: f64, scale_y: f64) -> Point {
        if i > 1 && j > 1 && i < self.height - 2 && j < self.width - 2 {
            let a = &self.get_node(i - 1, j).interp;
            let b = &self.get_node(i + 1, j).interp;
            let c = &self.get_node(i, j - 1).interp;
            let d = &self.get_node(i, j + 1).interp;
            let e = &self.get_node(i - 1, j - 1).interp;
            let f = &self.get_node(i + 1, j - 1).interp;
            let g = &self.get_node(i + 1, j + 1).interp;
            let h = &self.get_node(i - 1, j + 1).interp;
            let _i = &self.get_node(i - 2, j).interp;
            let _j = &self.get_node(i + 2, j).interp;
            let k = &self.get_node(i, j - 2).interp;
            let _l = &self.get_node(i, j + 2).interp;
            Point::new(
                (8. * (a.x + b.x + c.x + d.x)
                    - 2. * (e.x + f.x + g.x + h.x)
                    - (_i.x + _j.x + k.x + _l.x))
                    / 20.,
                (8. * (a.y + b.y + c.y + d.y)
                    - 2. * (e.y + f.y + g.y + h.y)
                    - (_i.y + _j.y + k.y + _l.y))
                    / 20.,
            )
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
            Point::new(sx / nb as f64, sy / nb as f64)
        }
    }
}
