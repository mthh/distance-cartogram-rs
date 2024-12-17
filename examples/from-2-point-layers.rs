use distance_cartogram::{get_nb_iterations, BBox, Grid, GridType};
use geo_types::Coord;
use geojson::{Feature, FeatureCollection, GeoJson, Geometry, Value};
use std::io::Write;
use std::time::Instant;

fn prepare_grid_geojson(
    grid: &Grid,
    grid_type: GridType,
    foreign_members: Option<geojson::JsonObject>,
) -> GeoJson {
    let mut features = Vec::new();
    for (i, polygon) in grid.get_grid(grid_type).iter().enumerate() {
        let geometry = Geometry::new(geojson::Value::from(polygon));
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

    let feature_collection = FeatureCollection {
        bbox: None,
        features,
        foreign_members,
    };
    GeoJson::FeatureCollection(feature_collection)
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

    // Read feature collection to get the crs of the layer if any
    let crs_background = read_crs(&geojson_background);

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

    // How much iterations to perform
    let n_iter = get_nb_iterations(points_source.len());

    // Actual grid computation
    let t = Instant::now();
    let grid = Grid::new(&points_source, &points_image, 2., n_iter, Some(bbox))
        .expect("Unable to create grid");
    println!(
        "Grid creation and initial interpolation step: {:?}",
        t.elapsed()
    );

    // Transform the background layer
    let t = Instant::now();
    let bg_transformed = grid
        .interpolate_layer(&bg)
        .expect("Unable to interpolate layer");
    println!("Layer interpolation: {:?}", t.elapsed());

    // Get the source grid and the interpolated grid...
    let grid_source = prepare_grid_geojson(&grid, GridType::Source, crs_background.clone());
    let grid_interpolated =
        prepare_grid_geojson(&grid, GridType::Interpolated, crs_background.clone());

    // ... and save them to files for latter visualization
    let mut file =
        std::fs::File::create("examples/grid-source.geojson").expect("Unable to create file");
    file.write_all(grid_source.to_string().as_bytes())
        .expect("Unable to write file grid-source.geojson");

    let mut file =
        std::fs::File::create("examples/grid-interpolated.geojson").expect("Unable to create file");
    file.write_all(grid_interpolated.to_string().as_bytes())
        .expect("Unable to write file grid-interpolated.geojson");

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
    let geojson = GeoJson::FeatureCollection(FeatureCollection {
        bbox: None,
        features,
        foreign_members: crs_background,
    });
    let mut file =
        std::fs::File::create("examples/data-transformed.geojson").expect("Unable to create file");
    file.write_all(geojson.to_string().as_bytes())
        .expect("Unable to write file data-transformed.geojson");
}
