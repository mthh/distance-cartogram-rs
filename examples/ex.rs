use std::io::Write;
use distance_cartogram::{Point, Grid, GridType};
use geojson::{Feature, FeatureCollection, GeoJson, Geometry, Value};

pub fn main() {
    let path_source = "examples/data-source-point.geojson";
    let path_image = "examples/data-image-point.geojson";
    let file_source = std::fs::File::open(path_source).expect("Unable to open file of image points");
    let file_image = std::fs::File::open(path_image).expect("Unable to open file of source points");

    let geojson_source = GeoJson::from_reader(&file_source).expect("Unable to read file of image points");
    let geojson_image = GeoJson::from_reader(&file_image).expect("Unable to read file of source points");

    let features_source = match geojson_source {
        GeoJson::FeatureCollection(collection) => collection.features,
        _ => panic!("Expected a feature collection"),
    };

    let features_images = match geojson_image {
        GeoJson::FeatureCollection(collection) => collection.features,
        _ => panic!("Expected a feature collection"),
    };

    let mut points_source = Vec::new();
    let mut points_image = Vec::new();

    for feature in features_source {
        let geometry = feature.geometry.unwrap();
        let coordinates = geometry.value;
        match coordinates {
            Value::Point(point) => {
                points_source.push(Point::new(point[0], point[1]));
            },
            _ => panic!("Expected a point"),
        }
    }

    for feature in features_images {
        let geometry = feature.geometry.unwrap();
        let coordinates = geometry.value;
        match coordinates {
            Value::Point(point) => {
                points_image.push(Point::new(point[0], point[1]));
            },
            _ => panic!("Expected a point"),
        }
    }

    if points_source.len() != points_image.len() {
        panic!("The number of source points and image points must be the same");
    }

    let mut grid = Grid::new(&points_source, 2.);
    let n_iter = (4. * (points_source.len() as f64).sqrt()).round() as usize;
    let interp_points = grid.interpolate(&points_image, n_iter);
    let grid_interpolated = grid.get_grid(GridType::Interpolated);

    // Convert it to GeoJson
    let mut features = Vec::new();
    for poly_coords in grid_interpolated {
        let mut coords = Vec::new();
        for coord in poly_coords {
            coords.push(vec![coord.x, coord.y]);
        }
        let geometry = Geometry::new(Value::Polygon(vec![coords]));
        let feature = Feature {
            bbox: None,
            geometry: Some(geometry),
            id: None,
            properties: None,
            foreign_members: None,
        };
        features.push(feature);
    }

    // Write the GeoJson to a file
    let feature_collection = FeatureCollection {
        bbox: None,
        features,
        foreign_members: None,
    };
    let geojson = GeoJson::FeatureCollection(feature_collection);
    let mut file = std::fs::File::create("examples/data-interpolated.geojson").expect("Unable to create file");
    let _ = file.write(geojson.to_string().as_bytes());
}