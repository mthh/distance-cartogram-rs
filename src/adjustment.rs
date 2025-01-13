use crate::errors::Error;
use geo_types::Coord;

pub enum AdjustmentType {
    Affine,
    Euclidean,
}

fn getScale(scale_x: f64, shear_x: f64, scale_y: f64, shear_y: f64) -> f64 {
    let scale_x0 = if shear_x == 0.0 {
        scale_x.abs()
    } else if scale_x == 0.0 {
        shear_x.abs()
    } else {
        (scale_x * scale_x + shear_x * shear_x).sqrt()
    };

    let scale_y0 = if shear_y == 0.0 {
        scale_y.abs()
    } else if scale_y == 0.0 {
        shear_y.abs()
    } else {
        (scale_y * scale_y + shear_y * shear_y).sqrt()
    };

    0.5 * (scale_x0 + scale_y0)
}

fn getRotation(scale_x: f64, shear_x: f64, scale_y: f64, shear_y: f64) -> f64 {
    let scale_x0 = if shear_x == 0.0 {
        scale_x.abs()
    } else if scale_x == 0.0 {
        shear_x.abs()
    } else {
        (scale_x * scale_x + shear_x * shear_x).sqrt()
    };

    let scale_y0 = if shear_y == 0.0 {
        scale_y.abs()
    } else if scale_y == 0.0 {
        shear_y.abs()
    } else {
        (scale_y * scale_y + shear_y * shear_y).sqrt()
    };

    (shear_y / scale_y0 - shear_x / scale_x0).atan2(scale_y / scale_y0 + scale_x / scale_x0)
}

pub struct TransformationMatrix {
    pub a11: f64,
    pub a12: f64,
    pub a13: f64,
    pub a21: f64,
    pub a22: f64,
    pub a23: f64,
    pub scale: f64,
    pub angle: f64,
    pub points_adjusted: Vec<Coord>,
}

impl std::fmt::Debug for TransformationMatrix {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TransformationMatrix")
            .field("a11", &self.a11)
            .field("a12", &self.a12)
            .field("a13", &self.a13)
            .field("a21", &self.a21)
            .field("a22", &self.a22)
            .field("a23", &self.a23)
            .field("scale", &self.scale)
            .field("angle", &self.angle)
            .finish()
    }
}

pub fn adjust(
    source_points: &[Coord],
    image_points: &[Coord],
    adjustment_type: AdjustmentType,
) -> Result<TransformationMatrix, Error> {
    let source_pts: Vec<_> = source_points.iter().map(|p| (p.x, p.y)).collect();
    let image_pts: Vec<_> = image_points.iter().map(|p| (p.x, p.y)).collect();

    let n = source_pts.len();

    if n != image_pts.len() {
        return Err(Error::InvalidInputPointsLength);
    }

    // Compute mean to center the points
    let mut src_mean_x = 0.0;
    let mut src_mean_y = 0.0;
    let mut img_mean_x = 0.0;
    let mut img_mean_y = 0.0;

    for (src, img) in source_pts.iter().zip(image_pts.iter()) {
        src_mean_x += src.0;
        src_mean_y += src.1;
        img_mean_x += img.0;
        img_mean_y += img.1;
    }

    src_mean_x /= n as f64;
    src_mean_y /= n as f64;
    img_mean_x /= n as f64;
    img_mean_y /= n as f64;

    // Compute adjustment coefficients
    let (a11, a12, a13, a21, a22, a23) = match adjustment_type {
        AdjustmentType::Euclidean => {
            let mut num1 = 0.0;
            let mut num2 = 0.0;
            let mut denom = 0.0;

            for (src, img) in source_pts.iter().zip(image_pts.iter()) {
                num1 += (src.0 - src_mean_x) * (img.0 - img_mean_x)
                    + (src.1 - src_mean_y) * (img.1 - img_mean_y);
                num2 += (src.0 - src_mean_x) * (img.1 - img_mean_y)
                    - (src.1 - src_mean_y) * (img.0 - img_mean_x);
                denom += (img.0 - img_mean_x).powi(2) + (img.1 - img_mean_y).powi(2);
            }

            let a11 = num1 / denom;
            let a12 = num2 / denom;
            let a21 = -a12;
            let a22 = a11;
            let a13 = src_mean_x - a11 * img_mean_x - a12 * img_mean_y;
            let a23 = src_mean_y - a21 * img_mean_x - a22 * img_mean_y;
            (a11, a12, a13, a21, a22, a23)
        }
        AdjustmentType::Affine => {
            let mut u2 = 0.0;
            let mut v2 = 0.0;
            let mut uv = 0.0;
            let mut xu = 0.0;
            let mut xv = 0.0;
            let mut yu = 0.0;
            let mut yv = 0.0;

            for (src, img) in source_pts.iter().zip(image_pts.iter()) {
                let u = img.0 - img_mean_x;
                let v = img.1 - img_mean_y;
                let x = src.0 - src_mean_x;
                let y = src.1 - src_mean_y;
                u2 += u * u;
                v2 += v * v;
                uv += u * v;
                xu += x * u;
                xv += x * v;
                yu += y * u;
                yv += y * v;
            }

            let denom = uv.powi(2) - u2 * v2;
            let a11 = (uv * xv - v2 * xu) / denom;
            let a12 = (uv * xu - u2 * xv) / denom;
            let a21 = (uv * yv - v2 * yu) / denom;
            let a22 = (uv * yu - u2 * yv) / denom;
            let a13 = src_mean_x - a11 * img_mean_x - a12 * img_mean_y;
            let a23 = src_mean_y - a21 * img_mean_x - a22 * img_mean_y;
            (a11, a12, a13, a21, a22, a23)
        }
    };

    // Compute adjusted points
    let adjusted_points = image_pts
        .iter()
        .map(|(cx, cy)| Coord {
            x: cx * a11 + cy * a12 + a13,
            y: cx * a21 + cy * a22 + a23,
        })
        .collect();

    let scale = getScale(a11, a12, a22, a21);
    let angle = getRotation(a11, a12, a22, a21).to_degrees();

    Ok(TransformationMatrix {
        a11,
        a12,
        a13,
        a21,
        a22,
        a23,
        scale,
        angle,
        points_adjusted: adjusted_points,
    })
}
