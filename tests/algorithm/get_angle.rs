use lazy_static::lazy_static;
use navdata_components::algorithm::measurement::get_angle;
use navdata_components::coordinate::Coordinate;

lazy_static! {
    static ref RAW: Coordinate = Coordinate {
        lat: 40.0,
        lon: 116.0
    };
    static ref TARGET: Coordinate = Coordinate {
        lat: 41.25,
        lon: 114.365
    };
}

#[test]
fn test1() {
    assert_eq!(get_angle(&RAW, &TARGET, 0.0), 314.9431579874174);
}

#[test]
fn test2() {
    assert_eq!(get_angle(&TARGET, &RAW, 0.0), 135.4793355143595);
}

#[test]
fn test3() {
    assert_eq!(get_angle(&TARGET, &RAW, 15.0), 120.47933551435949);
}
