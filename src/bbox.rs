/// Bounding box, defined by its minimum and maximum coordinates.
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
