use geo_types::Coord;

/// Compute the centroid of a set of points.
fn centroid(points: &[Coord]) -> Coord {
    let n = points.len() as f64;
    let sum_x = points.iter().map(|p| p.x).sum::<f64>();
    let sum_y = points.iter().map(|p| p.y).sum::<f64>();
    Coord {
        x: sum_x / n,
        y: sum_y / n,
    }
}

/// Center the points around the centroid.
fn center_points(points: &[Coord], centroid: Coord) -> Vec<Coord> {
    points
        .iter()
        .map(|p| Coord {
            x: p.x - centroid.x,
            y: p.y - centroid.y,
        })
        .collect()
}

/// Compute the norm of a set of points.
fn norm(points: &[Coord]) -> f64 {
    points
        .iter()
        .map(|p| p.x * p.x + p.y * p.y)
        .sum::<f64>()
        .sqrt()
}

/// Scale the points to a given norm.
fn scale_points(points: &[Coord], norm: f64) -> Vec<Coord> {
    points
        .iter()
        .map(|p| Coord {
            x: p.x / norm,
            y: p.y / norm,
        })
        .collect()
}

/// Compute the optimal rotation angle between two sets of points.
fn optimal_rotation(points1: &[Coord], points2: &[Coord]) -> f64 {
    let mut a = 0.0;
    let mut b = 0.0;
    for (p1, p2) in points1.iter().zip(points2.iter()) {
        a += p1.x * p2.x + p1.y * p2.y;
        b += p1.x * p2.y - p1.y * p2.x;
    }
    b.atan2(a)
}

/// Rotate a set of points by a given angle.
fn rotate_points(points: &[Coord], angle: f64) -> Vec<Coord> {
    points
        .iter()
        .map(|p| Coord {
            x: p.x * angle.cos() - p.y * angle.sin(),
            y: p.x * angle.sin() + p.y * angle.cos(),
        })
        .collect()
}

/// Compute the Procrustes distance between two sets of points
/// (Cf. https://en.wikipedia.org/wiki/Procrustes_analysis#Shape_comparison - here
/// we don't take the square root of the sum of the squared distances to avoid
/// the square root operation because we only need to compare the distances in a
/// first step).
fn procrustes_distance<'a>(zip_iter: impl Iterator<Item = (&'a Coord, &'a Coord)>) -> f64 {
    zip_iter
        .map(|(p1, p2): (&Coord, &Coord)| (p1.x - p2.x).powi(2) + (p1.y - p2.y).powi(2))
        .sum::<f64>()
}

pub struct ProcrustesResult {
    pub points: Vec<Coord>,
    pub angle: f64,
    pub scale: f64,
    pub translation: Coord,
    pub error: f64,
}

/// Apply the Procrustes analysis to two sets of points and return the transformed points
/// as well as the transformation parameters.
///
/// This is a naive version of the ordinary/classical Procrustes analysis (as described on
/// https://en.wikipedia.org/wiki/Procrustes_analysis#Ordinary_Procrustes_analysis) that only
/// deals with translation, rotation and scaling (i.e. it does not handle reflection yet).
pub(crate) fn procrustes(points1: &[Coord], points2: &[Coord]) -> ProcrustesResult {
    assert_eq!(
        points1.len(),
        points2.len(),
        "The two sets of points must have the same size"
    );

    // Compute the centroid of each set of points
    // and center the points around the centroid
    let centroid1 = centroid(points1);
    let centroid2 = centroid(points2);
    let centered1 = center_points(points1, centroid1);
    let centered2 = center_points(points2, centroid2);

    // Scale the points to a given norm
    let norm1 = norm(&centered1);
    let norm2 = norm(&centered2);
    let scaled1 = scale_points(&centered1, norm1);
    let scaled2 = scale_points(&centered2, norm2);

    // Compute the optimal rotation angle between the two sets of points
    let angle = optimal_rotation(&scaled1, &scaled2);
    let rotated2 = rotate_points(&scaled2, angle);

    // Check correspondence and reverse angle if necessary
    let rotated2_flipped = rotate_points(&scaled2, -angle);

    // Compute the error (aka the Procrustes distance,
    // cf. https://en.wikipedia.org/wiki/Procrustes_analysis#Shape_comparison)
    // for the two possible rotations in order to choose the best one
    let error_original = procrustes_distance(scaled1.iter().zip(rotated2.iter()));
    let error_flipped = procrustes_distance(scaled1.iter().zip(rotated2_flipped.iter()));

    // Choose the best rotation and finish the computation
    // of the error by taking the square root of the sum of the squared distances
    let (final_rotated2, error) = if error_flipped < error_original {
        (rotated2_flipped, error_flipped.sqrt())
    } else {
        (rotated2, error_original.sqrt())
    };

    // Final scaling and centering
    let pts = final_rotated2
        .iter()
        .map(|p| Coord {
            x: p.x * norm1 + centroid1.x,
            y: p.y * norm1 + centroid1.y,
        })
        .collect();

    ProcrustesResult {
        points: pts,
        angle,
        scale: norm1 / norm2,
        translation: Coord {
            x: centroid1.x - centroid2.x,
            y: centroid1.y - centroid2.y,
        },
        error,
    }
}
