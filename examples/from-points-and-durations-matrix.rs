use distance_cartogram::{generate_positions_from_durations, utils, BBox, Grid};
use geo_types::Coord;
use geojson::{Feature, FeatureCollection, GeoJson, Geometry, Value};
use std::io::Write;
use std::time::Instant;

fn main() {
    // The layer of source points (to be moved to create image points)
    let path_source = "examples/data-source-point.geojson";
    // The layer to deform (the background layer of territorial units)
    let path_layer_to_deform = "examples/background.geojson";
    // The durations matrix
    let path_durations = "./examples/mat.csv";

    let geojson_source = read_geojson(path_source);
    let geojson_background = read_geojson(path_layer_to_deform);

    let (durations, _ids) = utils::read_csv(
        std::fs::File::open(path_durations).expect("Unable to open file of durations"),
    );

    // Read the background layer.
    // We want to read the foreign members of the GeoJson FeatureCollection
    // to extract the CRS of the layer if any.
    // Read the CRS of the background layer if any
    let fm = read_crs(&geojson_background);

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

    let t = Instant::now();
    let positioning_result = generate_positions_from_durations(durations.clone(), &points_source)
        .expect("Unable to generate positions from durations");
    println!("Generating points from durations matrix: {:?}", t.elapsed());
    println!(
        "  ↳ Rotation angle: {}, Scale: {}, Translation: [{}, {}], Error (procrustes distance): {}",
        positioning_result.angle,
        positioning_result.scale,
        positioning_result.translation.x,
        positioning_result.translation.y,
        positioning_result.error,
    );
    let points_image = positioning_result.points;

    let feature_collection = create_fc_from_coords(&points_image, fm.clone());
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
    println!("BBox computation: {:?}", t.elapsed());

    // Prepare the grid for the cartogram
    // How many iterations to perform
    let n_iter = utils::get_nb_iterations(points_source.len());

    // Actual grid computation
    let t = Instant::now();
    let grid = Grid::new(&points_source, &points_image, 2., n_iter, Some(bbox))
        .expect("Unable to create grid");
    println!(
        "Grid creation, bidimensional regression step and metrics computation: {:?}",
        t.elapsed()
    );
    println!(
        "  ↳ MAE: {}, RMSE: {}, R-squared: {}, Deformation strength: {}",
        grid.mae(),
        grid.rmse(),
        grid.r_squared(),
        grid.deformation_strength()
    );

    // Transform the background layer
    let t = Instant::now();
    let bg_transformed = grid
        .interpolate_layer(&bg)
        .expect("Unable to interpolate layer");
    println!("Layer interpolation: {:?}", t.elapsed());

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
        foreign_members: fm,
    };

    save_to_file(&fc, "examples/data-transformed.geojson");
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

fn read_geojson(path: &str) -> GeoJson {
    let file_source =
        std::fs::File::open(path).expect(format!("Unable to open file '{}'", path).as_str());

    GeoJson::from_reader(&file_source).expect(format!("Unable to read file '{}'", path).as_str())
}

fn read_crs(geojson_layer: &GeoJson) -> Option<geojson::JsonObject> {
    let foreign_members = match &geojson_layer {
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
}
