use distance_cartogram::{Grid, GridType};
use geo_types::Coord;
use geojson::{Feature, FeatureCollection, GeoJson, Geometry, Value};
use std::io::Write;

pub fn main() {
    let path_source = "examples/data-source-point.geojson";
    let path_image = "examples/data-image-point.geojson";
    let path_layer_to_deform = "examples/background.geojson";

    let file_source =
        std::fs::File::open(path_source).expect("Unable to open file of image points");
    let file_image = std::fs::File::open(path_image).expect("Unable to open file of source points");
    let file_background =
        std::fs::File::open(path_layer_to_deform).expect("Unable to open file of layer to deform");

    let geojson_source =
        GeoJson::from_reader(&file_source).expect("Unable to read file of image points");
    let geojson_image =
        GeoJson::from_reader(&file_image).expect("Unable to read file of source points");
    let geojson_background =
        GeoJson::from_reader(&file_background).expect("Unable to read file of layer to deform");

    let features_source = match geojson_source {
        GeoJson::FeatureCollection(collection) => collection.features,
        _ => panic!("Expected a feature collection"),
    };

    let features_images = match geojson_image {
        GeoJson::FeatureCollection(collection) => collection.features,
        _ => panic!("Expected a feature collection"),
    };

    // We want to read the foreign members of the GeoJson FeatureCollection
    let crs_background = match &geojson_background {
        GeoJson::FeatureCollection(collection) => (*collection)
            .foreign_members.as_ref()
            .expect("No foreign members found")
            .get("crs")
            .expect("No crs found")
            .clone(),
        _ => panic!("Expected a feature collection"),
    };

    let mut fm = geojson::JsonObject::new();
    fm.insert("crs".to_string(), crs_background);

    let features_background = match geojson_background {
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
                points_source.push(Coord {
                    x: point[0],
                    y: point[1],
                });
            }
            _ => panic!("Expected a point"),
        }
    }

    for feature in features_images {
        let geometry = feature.geometry.unwrap();
        let coordinates = geometry.value;
        match coordinates {
            Value::Point(point) => {
                points_image.push(Coord {
                    x: point[0],
                    y: point[1],
                });
            }
            _ => panic!("Expected a point"),
        }
    }

    if points_source.len() != points_image.len() {
        panic!("The number of source points and image points must be the same");
    }

    let bg: Vec<geo_types::Geometry> = features_background
        .into_iter()
        .map(|feature_geojson| geo_types::Geometry::<f64>::try_from(feature_geojson).unwrap())
        .collect::<Vec<_>>();

    // Compute BBox of bg
    let mut xmin = f64::INFINITY;
    let mut ymin = f64::INFINITY;
    let mut xmax = f64::NEG_INFINITY;
    let mut ymax = f64::NEG_INFINITY;

    bg.iter().for_each(|f| match f {
        geo_types::Geometry::Polygon(p) => {
            p.exterior().0.iter().for_each(|c| {
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
            });
        }
        geo_types::Geometry::MultiPolygon(mp) => {
            mp.iter().for_each(|p| {
                p.exterior().0.iter().for_each(|c| {
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
                });
            });
        }
        _ => panic!("Only Polygon and MultiPolygon are supported"),
    });

    let mut grid = Grid::new(&points_source, 2., Some((xmin, ymin, xmax, ymax).into()));
    let n_iter = (4. * (points_source.len() as f64).sqrt()).round() as usize;
    let interp_points = grid.interpolate(&points_image, n_iter);

    // Get the source grid
    let grid_source = grid.get_grid(GridType::Source);

    // Get the interpolated grid
    let grid_interpolated = grid.get_grid(GridType::Interpolated);

    let mut features = Vec::new();
    for polygon in grid_source {
        let geometry = Geometry::new(geojson::Value::from(&polygon));
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
        foreign_members: Some(fm.clone()),
    };
    let geojson = GeoJson::FeatureCollection(feature_collection);
    let mut file =
        std::fs::File::create("examples/grid-source.geojson").expect("Unable to create file");
    let _ = file.write(geojson.to_string().as_bytes());

    // Convert interpolated grid to GeoJson
    let mut features = Vec::new();
    for polygon in grid_interpolated {
        let geometry = Geometry::new(geojson::Value::from(&polygon));
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
        foreign_members: Some(fm.clone()),
    };
    let geojson = GeoJson::FeatureCollection(feature_collection);
    let mut file =
        std::fs::File::create("examples/grid-interpolated.geojson").expect("Unable to create file");
    let _ = file.write(geojson.to_string().as_bytes());

    // Transform the background layer
    let bg_transformed = grid.interpolate_layer(&bg);

    // Write the GeoJson to a file
    let mut features = Vec::new();
    for polygon in bg_transformed {
        let geometry = Geometry::new(geojson::Value::from(&polygon));
        let feature = Feature {
            bbox: None,
            geometry: Some(geometry),
            id: None,
            properties: None,
            foreign_members: None,
        };
        features.push(feature);
    }
    let geojson = GeoJson::FeatureCollection(FeatureCollection {
        bbox: None,
        features,
        foreign_members: Some(fm),
    });
    let mut file =
        std::fs::File::create("examples/data-transformed.geojson").expect("Unable to create file");
    let _ = file.write(geojson.to_string().as_bytes());
}
