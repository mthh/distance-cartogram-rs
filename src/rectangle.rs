use crate::bbox::BBox;
use crate::point::Point;

/// A 2D rectangle, defined by a point (x, y) and dimension (width x height).
#[derive(Debug)]
pub(crate) struct Rectangle2D {
    x: f64,
    y: f64,
    height: f64,
    width: f64,
}

impl Rectangle2D {
    /// Create a new rectangle.
    pub fn new(x: f64, y: f64, height: f64, width: f64) -> Rectangle2D {
        Rectangle2D {
            x,
            y,
            height,
            width,
        }
    }

    /// Add a point to the rectangle.
    pub fn add(&mut self, pt: &Point) {
        if pt.x < self.x {
            self.width += self.x - pt.x;
            self.x = pt.x;
        } else if pt.x > self.x + self.width {
            self.width = pt.x - self.x;
        }
        if pt.y < self.y {
            self.height += self.y - pt.y;
            self.y = pt.y;
        } else if pt.y > self.y + self.height {
            self.height = pt.y - self.y;
        }
    }

    /// Update the rectangle from a center and a corner.
    pub fn set_rect_from_center(&mut self, center: &Point, corner: &Point) {
        self.x = center.x - (corner.x - center.x).abs();
        self.y = center.y - (corner.y - center.y).abs();
        self.width = (corner.x - center.x).abs() * 2.0;
        self.height = (corner.y - center.y).abs() * 2.0;
    }

    /// Update the rectangle from a bounding box.
    pub fn set_from_bbox(&mut self, bbox: &BBox) {
        self.x = bbox.xmin;
        self.y = bbox.ymin;
        self.width = bbox.xmax - bbox.xmin;
        self.height = bbox.ymax - bbox.ymin;
    }

    pub fn center_x(&self) -> f64 {
        self.x + self.width / 2.0
    }

    pub fn center_y(&self) -> f64 {
        self.y + self.height / 2.0
    }

    pub fn min_x(&self) -> f64 {
        self.x
    }

    pub fn max_x(&self) -> f64 {
        self.x + self.width
    }

    pub fn min_y(&self) -> f64 {
        self.y
    }

    pub fn max_y(&self) -> f64 {
        self.y + self.height
    }

    pub fn height(&self) -> f64 {
        self.height
    }

    pub fn width(&self) -> f64 {
        self.width
    }

    /// Create a Rectangle2D from a list of points.
    pub fn from_points(points: &[Point]) -> Rectangle2D {
        if points.is_empty() {
            return Rectangle2D::new(0.0, 0.0, 0.0, 0.0);
        }
        let mut rect = Rectangle2D::new(points[0].x, points[0].y, 0.0, 0.0);
        for pt in points.iter().skip(1) {
            rect.add(pt);
        }
        rect
    }

    /// Create a Rectangle2D from a BBox.
    pub fn from_bbox(bbox: &BBox) -> Rectangle2D {
        Rectangle2D {
            x: bbox.xmin,
            y: bbox.ymin,
            width: bbox.xmax - bbox.xmin,
            height: bbox.ymax - bbox.ymin,
        }
    }

    /// Convert the Rectangle2D to a BBox.
    pub fn as_bbox(&self) -> BBox {
        BBox {
            xmin: self.x,
            xmax: self.x + self.width,
            ymin: self.y,
            ymax: self.y + self.height,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rectangle2d() {
        let mut rect = Rectangle2D::new(0.0, 0.0, 0.0, 0.0);
        let pt = Point::new(1.0, 1.0);
        rect.add(&pt);
        assert_eq!(rect.x, 0.0);
        assert_eq!(rect.y, 0.0);
        assert_eq!(rect.width, 1.0);
        assert_eq!(rect.height, 1.0);
        let pt = Point::new(-1.0, -1.0);
        rect.add(&pt);
        assert_eq!(rect.x, -1.0);
        assert_eq!(rect.y, -1.0);
        assert_eq!(rect.width, 2.0);
        assert_eq!(rect.height, 2.0);
    }

    #[test]
    fn test_as_bbox() {
        let mut rect = Rectangle2D::new(0.0, 0.0, 1.0, 1.0);
        rect.add(&Point::new(12.0, 22.0));
        rect.add(&Point::new(-3.0, -4.0));
        let bbox = rect.as_bbox();
        assert_eq!(bbox.xmin, -3.0);
        assert_eq!(bbox.ymin, -4.0);
        assert_eq!(bbox.xmax, 12.0);
        assert_eq!(bbox.ymax, 22.0);
    }

    #[test]
    fn test_from_bbox() {
        let bbox = BBox {
            xmin: -3.0,
            ymin: -4.0,
            xmax: 12.0,
            ymax: 22.0,
        };
        let rect = Rectangle2D::from_bbox(&bbox);
        assert_eq!(rect.x, -3.0);
        assert_eq!(rect.y, -4.0);
        assert_eq!(rect.width, 15.0);
        assert_eq!(rect.height, 26.0);
    }
}
