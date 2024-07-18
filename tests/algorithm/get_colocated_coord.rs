use lazy_static::lazy_static;
use navdata_components::algorithm::position_transformation::get_colocated_coord;
use navdata_components::coordinate::Coordinate;

lazy_static! {
    static ref RAW: Coordinate = Coordinate {
        lat: 40.0,
        lon: 116.0
    };
}

#[test]
fn test1() {
    assert_eq!(
        get_colocated_coord(&RAW, 112.0, 36502.0, 0.0),
        Coordinate {
            lat: 39.87617306290192,
            lon: 116.39561635066224
        }
    );
}
#[test]
fn test2() {
    assert_eq!(
        get_colocated_coord(&RAW, 232.0, 2354.5, 0.0),
        Coordinate {
            lat: 39.986942788836465,
            lon: 115.97827691463395
        }
    );
}

#[test]
fn test3() {
    assert_eq!(
        get_colocated_coord(&RAW, 232.0, 2354.5, 12.0),
        Coordinate {
            lat: 39.99070164156582,
            lon: 115.97522159435215
        }
    );
}
