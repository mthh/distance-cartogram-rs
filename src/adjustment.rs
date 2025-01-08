use crate::errors::Error;
use geo_types::Coord;

pub enum AdjustmentType {
    Affine,
    Euclidean,
}

pub struct TransformationMatrix {
    pub a11: f64,
    pub a12: f64,
    pub a13: f64,
    pub a21: f64,
    pub a22: f64,
    pub a23: f64,
    pub angle: f64,
    pub scale: f64,
    pub adjusted_points: Vec<Coord>,
}

pub fn adjust(
    source_points: &[Coord],
    image_points: &[Coord],
    adjustment_type: AdjustmentType,
) -> Result<TransformationMatrix, Error> {
    let source_points: Vec<_> = source_points.iter().map(|p| (p.x, p.y)).collect();
    let image_points: Vec<_> = image_points.iter().map(|p| (p.x, p.y)).collect();

    let n = source_points.len();

    if n != image_points.len() {
        return Err(Error::InvalidInputPointsLength);
    }

    // Compute mean to center the points
    let mut src_mean_x = 0.0;
    let mut src_mean_y = 0.0;
    let mut img_mean_x = 0.0;
    let mut img_mean_y = 0.0;

    for (src, img) in source_points.iter().zip(image_points.iter()) {
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
    let mut a11 = 0.0;
    let mut a12 = 0.0;
    let mut a13 = 0.0;
    let mut a21 = 0.0;
    let mut a22 = 0.0;
    let mut a23 = 0.0;

    match adjustment_type {
        AdjustmentType::Euclidean => {
            let mut num1 = 0.0;
            let mut num2 = 0.0;
            let mut denom = 0.0;

            for (src, img) in source_points.iter().zip(image_points.iter()) {
                num1 += (src.0 - src_mean_x) * (img.0 - img_mean_x)
                    + (src.1 - src_mean_y) * (img.1 - img_mean_y);
                num2 += (src.0 - src_mean_y) * (img.1 - img_mean_y)
                    - (src.1 - src_mean_x) * (img.0 - img_mean_x);
                denom += (img.0 - img_mean_x).powi(2)
                    + (img.1 - img_mean_y).powi(2);
            }

            a11 = num1 / denom;
            a12 = num2 / denom;
            a21 = -a12;
            a22 = a11;
            a13 = src_mean_x - a11 * img_mean_x - a12 * img_mean_y;
            a23 = src_mean_y - a21 * img_mean_x - a22 * img_mean_y;
        }
        AdjustmentType::Affine => {
            let mut u2 = 0.0;
            let mut v2 = 0.0;
            let mut uv = 0.0;
            let mut xu = 0.0;
            let mut xv = 0.0;
            let mut yu = 0.0;
            let mut yv = 0.0;

            for (src, img) in source_points.iter().zip(image_points.iter()) {
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
            a11 = (uv * xv - v2 * xu) / denom;
            a12 = (uv * xu - u2 * xv) / denom;
            a21 = (uv * yv - v2 * yu) / denom;
            a22 = (uv * yu - u2 * yv) / denom;
            a13 = src_mean_x - a11 * img_mean_x - a12 * img_mean_y;
            a23 = src_mean_y - a21 * img_mean_x - a22 * img_mean_y;
        }
    }

    // Compute adjusted points
    let adjusted_points = image_points
        .iter()
        .map(|(x, y)| {
            let x = a11 * x + a12 * y + a13;
            let y = a21 * x + a22 * y + a23;
            Coord { x, y }
        })
        .collect();

    let scale = ((a11 * a11 + a21 * a21) + (a12 * a12 + a22 * a22)).sqrt();
    let angle = a11.atan2(a21);

    Ok(TransformationMatrix {
        a11,
        a12,
        a13,
        a21,
        a22,
        a23,
        angle,
        scale,
        adjusted_points,
    })
}
