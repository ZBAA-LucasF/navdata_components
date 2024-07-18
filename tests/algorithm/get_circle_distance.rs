use lazy_static::lazy_static;
use navdata_components::algorithm::measurement::get_circle_distance;
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
    assert_eq!(get_circle_distance(&RAW, &TARGET), 196067.796222155);
}

#[test]
fn test2() {
    assert_eq!(get_circle_distance(&TARGET, &RAW), 196067.796222155);
}

#[test]
fn test3() {
    assert_eq!(
        get_circle_distance(&RAW, &TARGET),
        get_circle_distance(&TARGET, &RAW)
    );
}
