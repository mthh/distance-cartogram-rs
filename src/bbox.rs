use geo_types::Coord;

/// Bounding box, defined by its minimum and maximum coordinates,
/// used to control the extent of the interpolation grid (see [`Grid`](crate::Grid)).
#[derive(Debug)]
pub struct BBox {
    pub xmin: f64,
    pub ymin: f64,
    pub xmax: f64,
    pub ymax: f64,
}

impl From<(f64, f64, f64, f64)> for BBox {
    fn from(val: (f64, f64, f64, f64)) -> Self {
        BBox {
            xmin: val.0,
            ymin: val.1,
            xmax: val.2,
            ymax: val.3,
        }
    }
}

impl BBox {
    /// Create a new bounding box from its minimum and maximum coordinates.
    pub fn new(xmin: f64, ymin: f64, xmax: f64, ymax: f64) -> Self {
        BBox {
            xmin,
            ymin,
            xmax,
            ymax,
        }
    }

    /// Compute whether a point is inside the bounding box.
    pub fn contains(&self, point: &Coord) -> bool {
        point.x >= self.xmin && point.x <= self.xmax && point.y >= self.ymin && point.y <= self.ymax
    }

    /// Compute whether a bounding box is inside the bounding box.
    pub fn contains_bbox(&self, bbox: &BBox) -> bool {
        bbox.xmin >= self.xmin
            && bbox.xmax <= self.xmax
            && bbox.ymin >= self.ymin
            && bbox.ymax <= self.ymax
    }

    pub fn from_geometries(geometries: &[geo_types::Geometry]) -> Self {
        let mut xmin = f64::INFINITY;
        let mut ymin = f64::INFINITY;
        let mut xmax = f64::NEG_INFINITY;
        let mut ymax = f64::NEG_INFINITY;

        let mut box_coord = |c: &Coord| {
            if c.x < xmin {
                xmin = c.x;
            }
            if c.x > xmax {
                xmax = c.x;
            }
            if c.y < ymin {
                ymin = c.y;
            }
            if c.y > ymax {
                ymax = c.y;
            }
        };

        geometries.iter().for_each(|f| match f {
            geo_types::Geometry::Point(p) => {
                box_coord(&p.0);
            }
            geo_types::Geometry::MultiPoint(mp) => {
                mp.iter().for_each(|p| box_coord(&p.0));
            }
            geo_types::Geometry::LineString(l) => {
                l.0.iter().for_each(&mut box_coord);
            }
            geo_types::Geometry::MultiLineString(mls) => {
                mls.iter().for_each(|l| l.0.iter().for_each(&mut box_coord))
            }
            geo_types::Geometry::Polygon(p) => {
                p.exterior().0.iter().for_each(&mut box_coord);
            }
            geo_types::Geometry::MultiPolygon(mp) => {
                mp.iter().for_each(|p| {
                    p.exterior().0.iter().for_each(&mut box_coord);
                });
            }
            geo_types::Geometry::Triangle(t) => {
                box_coord(&t.0);
                box_coord(&t.1);
                box_coord(&t.2);
            }
            geo_types::Geometry::Rect(r) => {
                box_coord(&r.min());
                box_coord(&r.max());
            }
            geo_types::Geometry::Line(l) => {
                box_coord(&l.start);
                box_coord(&l.end);
            }
            geo_types::Geometry::GeometryCollection(gc) => {
                let bb = BBox::from_geometries(&gc.0);
                box_coord(&Coord {
                    x: bb.xmin,
                    y: bb.ymin,
                });
                box_coord(&Coord {
                    x: bb.xmax,
                    y: bb.ymax,
                });
            }
        });
        BBox::new(xmin, ymin, xmax, ymax)
    }
}
