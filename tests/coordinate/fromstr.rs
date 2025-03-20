use std::str::FromStr;

use navdata_components::coordinate::{CoordParseError, Coordinate};

#[allow(non_snake_case)]
#[test]
fn test_fromstr_NxxxxxxExxxxxxx_1() {
    let result = Coordinate::from_str("N400000E1160000").unwrap();

    assert_eq!(
        result,
        Coordinate {
            lat: 40.0,
            lon: 116.0
        }
    );
}

#[allow(non_snake_case)]
#[test]
fn test_fromstr_NxxxxxxExxxxxxx_2() {
    let result = Coordinate::from_str("N403000E1163000").unwrap();

    assert_eq!(
        result,
        Coordinate {
            lat: 40.5,
            lon: 116.5
        }
    );
}

#[allow(non_snake_case)]
#[test]
fn test_fromstr_NxxxxxxExxxxxxx_3() {
    let result = Coordinate::from_str("N403036E1163036").unwrap();

    assert_eq!(
        result,
        Coordinate {
            lat: 40.51,
            lon: 116.51
        }
    );
}

#[allow(non_snake_case)]
#[test]
fn test_fromstr_Nxxxx_xExxxx_x_1() {
    let result = Coordinate::from_str("N400000.0E1160000.0").unwrap();

    assert_eq!(
        result,
        Coordinate {
            lat: 40.0,
            lon: 116.0
        }
    );
}
#[allow(non_snake_case)]
#[test]
fn test_fromstr_Nxxxx_xExxxx_x_2() {
    let result = Coordinate::from_str("N403000.0E1163000.0").unwrap();

    assert_eq!(
        result,
        Coordinate {
            lat: 40.5,
            lon: 116.5
        }
    );
}

#[allow(non_snake_case)]
#[test]
fn test_fromstr_Nxxxx_xExxxx_x_3() {
    let result = Coordinate::from_str("N403003.6E1163003.6").unwrap();

    assert_eq!(
        result,
        Coordinate {
            lat: 40.501,
            lon: 116.501
        }
    );
}

#[allow(non_snake_case)]
#[test]
fn test_fromstr_ES_1() {
    let result = Coordinate::from_str("N040.00.00.000 E116.00.00.000").unwrap();

    assert_eq!(
        result,
        Coordinate {
            lat: 40.0,
            lon: 116.0
        }
    );
}

#[allow(non_snake_case)]
#[test]
fn test_fromstr_ES_2() {
    let result = Coordinate::from_str("N040.30.00.000 E116.30.00.000").unwrap();

    assert_eq!(
        result,
        Coordinate {
            lat: 40.5,
            lon: 116.5
        }
    );
}

#[allow(non_snake_case)]
#[test]
fn test_fromstr_ES_3() {
    let result = Coordinate::from_str("N040.30.03.600 E116.30.03.600").unwrap();

    assert_eq!(
        result,
        Coordinate {
            lat: 40.501,
            lon: 116.501
        }
    );
}

#[test]
fn test_fromstr_pmdg_1() {
    let result = Coordinate::from_str("N 40 00.0 E 116 00.0").unwrap();

    assert_eq!(
        result,
        Coordinate {
            lat: 40.0,
            lon: 116.0
        }
    );
}

#[test]
fn test_fromstr_pmdg_2() {
    let result = Coordinate::from_str("N 40 30.0 E 116 30.0").unwrap();

    assert_eq!(
        result,
        Coordinate {
            lat: 40.5,
            lon: 116.5
        }
    );
}

#[test]
fn test_fromstr_pmdg_3() {
    let result = Coordinate::from_str("N 40 30.6 E 116 30.6").unwrap();

    assert_eq!(
        result,
        Coordinate {
            lat: 40.51,
            lon: 116.51
        }
    );
}

#[test]
fn test_fromstr_xx_1() {
    let result = Coordinate::from_str("40.0,116.0").unwrap();

    assert_eq!(
        result,
        Coordinate {
            lat: 40.0,
            lon: 116.0
        }
    );
}

#[test]
fn test_fromstr_xx_2() {
    let result = Coordinate::from_str("40.123456,116.654321").unwrap();

    assert_eq!(
        result,
        Coordinate {
            lat: 40.123456,
            lon: 116.654321
        }
    );
}

#[test]
fn test_lat_error() {
    let result = Coordinate::from_str("40.12l3456,116.654321");
    assert_eq!(result, Err(CoordParseError::LatParseError));
}

#[test]
fn test_lon_error() {
    let result = Coordinate::from_str("40.123456,116.6543l21");
    assert_eq!(result, Err(CoordParseError::LonParseError));
}
