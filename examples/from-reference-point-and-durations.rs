use distance_cartogram::{concentric_circles, move_points, utils, BBox, CentralTendency, Grid};
use geo_types::Coord;
use geojson::{Feature, FeatureCollection, GeoJson, Geometry, Value};
use std::io::Write;
use std::time::Instant;

fn main() {
    // The layer of source points (to be moved to create image points)
    let path_source = "examples/data-source-point.geojson";
    // The layer to deform (the background layer of territorial units)
    let path_layer_to_deform = "examples/background.geojson";

    let file_source =
        std::fs::File::open(path_source).expect("Unable to open file of source points");
    let file_background =
        std::fs::File::open(path_layer_to_deform).expect("Unable to open file of layer to deform");

    let geojson_source =
        GeoJson::from_reader(&file_source).expect("Unable to read file of source points");
    let geojson_background =
        GeoJson::from_reader(&file_background).expect("Unable to read file of layer to deform");

    // Read the background layer.
    // We want to read the foreign members of the GeoJson FeatureCollection
    // to extract the CRS of the layer if any.
    let fm = {
        let foreign_members = match &geojson_background {
            GeoJson::FeatureCollection(collection) => collection.foreign_members.as_ref(),
            _ => panic!("Expected a feature collection"),
        };
        if let Some(foreign_members) = foreign_members {
            if foreign_members.contains_key("crs") {
                let mut fm = geojson::JsonObject::new();
                fm.insert(
                    "crs".to_string(),
                    foreign_members.get("crs").unwrap().clone(),
                );
                Some(fm)
            } else {
                None
            }
        } else {
            None
        }
    };

    let features_background = match geojson_background {
        GeoJson::FeatureCollection(collection) => collection.features,
        _ => panic!("Expected a feature collection"),
    };

    // Extract source points to a Vec<Coord>
    let mut points_source = Vec::new();
    match geojson_source {
        GeoJson::FeatureCollection(collection) => {
            let fts = collection.features;
            for feature in fts {
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
        }
        _ => panic!("Expected a feature collection"),
    };

    // Those are the times associated with the points
    // They are ordered in the same way as the points
    // and describe the time it takes to go from the origin
    // points (the one with time 0) to the destination points
    // (all other points)
    let times = vec![
        311.7, 172.2, 413., 64., 352.3, 271.6, 257.1, 380.2, 369.1, 364.7, 574.4, 295.5, 110.5,
        429.3, 273.8, 142.3, 354.7, 387.7, 356.7, 187.7, 354.7, 540.6, 157.3, 358.7, 254.4, 309.7,
        401.9, 370.4, 332.8, 294.9, 415., 421.3, 190.7, 509.4, 161.7, 482.4, 369.3, 392.5, 342.1,
        333.1, 323.6, 440.8, 194., 498.5, 339.2, 446.9, 240.4, 452., 290., 265.3, 420.7, 324.2,
        356.4, 300.1, 118.3, 261.6, 494.4, 473.7, 317.4, 231.8, 108.7, 494.4, 110.3, 81.6, 505.,
        391.9, 318.8, 351.7, 193., 475.9, 73.5, 417.7, 658.8, 0., 149.3, 465.8, 221., 335.4, 292.6,
        356.1, 443., 244.4, 206., 433.6, 607.5, 315.2, 99.7, 42.4, 357.6, 514.6, 417.1, 334.,
        381.6, 539.9,
    ];

    let t = Instant::now();
    let positioning_result =
        move_points(&points_source, &times, 1., CentralTendency::Median).unwrap();
    println!("Moving points: {:?}", t.elapsed());
    println!(
        "  ↳ Reference speed: {:?} km/h",
        positioning_result.reference_speed / 1000. * 60.
    );
    let points_image = &positioning_result.points;
    let feature_collection = create_fc_from_coords(points_image, fm.clone());

    save_to_file(&feature_collection, "examples/moved-points.geojson");

    // Read the background layer
    // Extract properties and geometries from the background layer
    let mut props_bg_layer = Vec::with_capacity(features_background.len());
    let bg: Vec<geo_types::Geometry> = features_background
        .into_iter()
        .map(|feature_geojson| {
            props_bg_layer.push(feature_geojson.properties.clone());
            geo_types::Geometry::<f64>::try_from(feature_geojson).unwrap()
        })
        .collect::<Vec<_>>();

    // Compute BBox of background layer
    let t = Instant::now();
    let bbox = BBox::from_geometries(&bg);
    println!("\nBBox computation: {:?}", t.elapsed());

    // Prepare the grid for the cartogram
    // How many iterations to perform
    let n_iter = utils::get_nb_iterations(points_source.len());

    // Actual grid computation
    let t = Instant::now();
    let grid = Grid::new(&points_source, points_image, 2., n_iter, Some(bbox))
        .expect("Unable to create grid");
    println!(
        "\nGrid creation, bidimensional regression step and metrics computation: {:?}",
        t.elapsed()
    );
    println!(
        "  ↳ MAE: {}, RMSE (interpolated - image): {:?}, RMSE (interpolated - source): {:?}, R-squared: {}, Deformation strength: {}",
        grid.mae(),
        grid.rmse_interp_image(),
        grid.rmse_interp_source(),
        grid.r_squared(),
        grid.deformation_strength()
    );

    // Transform the background layer
    let t = Instant::now();
    let bg_transformed = grid
        .interpolate_layer(&bg)
        .expect("Unable to interpolate layer");
    println!("\nLayer interpolation: {:?}", t.elapsed());

    // Write the GeoJson to a file, taking care to transferring the original properties
    let mut features = Vec::new();
    for (polygon, props) in bg_transformed.into_iter().zip(props_bg_layer.into_iter()) {
        let geometry = Geometry::new(geojson::Value::from(&polygon));
        let feature = Feature {
            bbox: None,
            geometry: Some(geometry),
            id: None,
            properties: props,
            foreign_members: None,
        };
        features.push(feature);
    }
    let fc = FeatureCollection {
        bbox: None,
        features,
        foreign_members: fm.clone(),
    };

    save_to_file(&fc, "examples/data-transformed.geojson");

    // Also create concentric circles around the reference point
    // The durations are given in minutes so we pass the steps in minutes too.
    let t = Instant::now();
    let circles = concentric_circles(&positioning_result, vec![60., 180., 300., 420., 540., 660.]);
    println!("\nConcentric circles creation: {:?}", t.elapsed());

    let mut features = Vec::new();

    for (geometry, duration) in circles {
        let geom = geojson::Geometry::new(Value::from(&geometry));
        let mut props = geojson::JsonObject::new();
        props.insert("duration".to_string(), duration.into());
        let feature = Feature {
            bbox: None,
            geometry: Some(geom),
            id: None,
            properties: Some(props),
            foreign_members: None,
        };
        features.push(feature);
    }

    save_to_file(
        &FeatureCollection {
            bbox: None,
            features,
            foreign_members: fm,
        },
        "examples/concentric-circles.geojson",
    );
}

fn create_fc_from_coords(pts: &[Coord], fm: Option<geojson::JsonObject>) -> FeatureCollection {
    let mut features = Vec::new();

    for (i, pt) in pts.iter().enumerate() {
        let geometry = Geometry::new(Value::Point(vec![pt.x, pt.y]));
        let mut props = geojson::JsonObject::new();
        props.insert("id".to_string(), i.into());
        let feature = Feature {
            bbox: None,
            geometry: Some(geometry),
            id: None,
            properties: Some(props),
            foreign_members: None,
        };
        features.push(feature);
    }

    FeatureCollection {
        bbox: None,
        features,
        foreign_members: fm.clone(),
    }
}

fn save_to_file(feature_collection: &FeatureCollection, path: &str) {
    std::fs::File::create(path)
        .expect(format!("Unable to create file {}", path).as_str())
        .write_all(feature_collection.to_string().as_bytes())
        .expect(format!("Unable to write file {}", path).as_str());
}
