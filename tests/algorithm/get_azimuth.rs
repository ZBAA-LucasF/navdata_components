use navdata_components::algorithm::measurement::{get_azimuth, get_distance};
use navdata_components::algorithm::position_transformation::get_colocated_coord;
use navdata_components::coordinate::Coordinate;

#[test]
fn inverts_the_direct_solution() {
    let origin = Coordinate {
        lat: 37.4825,
        lon: 112.971_666_666_7,
    };
    for bearing in [0.0, 1.0, 45.0, 90.0, 135.5, 180.0, 270.0, 359.0] {
        for distance in [500.0, 5_500.0, 185_200.0] {
            let target = get_colocated_coord(&origin, bearing, distance, 0.0);
            let azimuth = get_azimuth(&origin, &target);
            let error = ((azimuth - bearing + 540.0) % 360.0 - 180.0).abs();
            assert!(error < 1e-7, "{bearing}° {distance} m -> {azimuth}");
            assert!((get_distance(&origin, &target) - distance).abs() < 1e-3);
        }
    }
    assert_eq!(get_azimuth(&origin, &origin), 0.0);
}

#[test]
fn crosses_the_antimeridian() {
    let east = Coordinate {
        lat: 0.0,
        lon: 179.9,
    };
    let west = Coordinate {
        lat: 0.0,
        lon: -179.9,
    };
    assert!((get_azimuth(&east, &west) - 90.0).abs() < 1e-6);
    assert!((get_azimuth(&west, &east) - 270.0).abs() < 1e-6);
}
