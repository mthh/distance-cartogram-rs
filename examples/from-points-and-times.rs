use distance_cartogram::{get_nb_iterations, move_points, CentralTendency, Grid, GridType};
use geo_types::Coord;
use geojson::{Feature, FeatureCollection, GeoJson, Geometry, Value};
use std::io::Write;
use std::time::Instant;

fn main() {
    let path_source = "examples/data-source-point.geojson";
    let path_layer_to_deform = "examples/background.geojson";

    let file_source =
        std::fs::File::open(path_source).expect("Unable to open file of image points");
    let file_background =
        std::fs::File::open(path_layer_to_deform).expect("Unable to open file of layer to deform");

    let geojson_source =
        GeoJson::from_reader(&file_source).expect("Unable to read file of image points");
    let geojson_background =
        GeoJson::from_reader(&file_background).expect("Unable to read file of layer to deform");

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
    let new_pts = move_points(&points_source, &times, 1., CentralTendency::Median).unwrap();
    println!("Moving points: {:?}", t.elapsed());

    // Save the displaced points to file
    let mut features = Vec::new();
    for (i, pt) in new_pts.iter().enumerate() {
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

    let feature_collection = FeatureCollection {
        bbox: None,
        features,
        foreign_members: None,
    };

    let mut file =
        std::fs::File::create("/tmp/moved_points.geojson").expect("Unable to create file");
    file.write_all(feature_collection.to_string().as_bytes())
        .expect("Unable to write file /tmp/moved_points.geojson");
}
