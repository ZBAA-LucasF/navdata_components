use navdata_components::coordinate::Coordinate;
use navdata_components::parse::{parse_flexible, AxisOrder, FlexibleParseError};

fn near(actual: f64, expected: f64, tolerance: f64) {
    assert!(
        (actual - expected).abs() <= tolerance,
        "{actual} != {expected}"
    );
}

#[test]
fn flexible_formats_and_hemispheres() {
    for text in [
        "39.9,116.4",
        "39.9，116.4",
        "39.9 116.4",
        "N395400E1162400",
        "N3954.000 E11624.000",
        "N395400.00 E1162400.00",
        "N39540000 E116240000",
        "N039.54.00.000 E116.24.00.000",
        "N39°54′00″ E116°24′00″",
        "39°54′00″N 116°24′00″E",
        "E1162400 N395400",
        "116.4E 39.9N",
        "N 39 54.0 E 116 24.0",
        "n395400e1162400",
    ] {
        let p = parse_flexible(text, AxisOrder::LatLon).unwrap_or_else(|e| panic!("{text}: {e}"));
        near(p.lat, 39.9, 1e-8);
        near(p.lon, 116.4, 1e-8);
    }
    let p = Coordinate::parse_flexible("116.4,39.9", AxisOrder::LonLat).unwrap();
    near(p.lat, 39.9, 1e-8);
    for (text, order) in [
        ("39 54 00 116 24 00", AxisOrder::LatLon),
        ("39 54.0 116 24.0", AxisOrder::LatLon),
        ("116 24 00 39 54 00", AxisOrder::LonLat),
        ("-39 54 00 116 24 00", AxisOrder::LatLon),
    ] {
        let p = parse_flexible(text, order).unwrap_or_else(|e| panic!("{text}: {e}"));
        near(p.lat.abs(), 39.9, 1e-8);
        near(p.lon, 116.4, 1e-8);
    }
    for text in [
        "-37.673,144.843",
        "S374022.8 E1445034.8",
        "37.673S,144.843E",
    ] {
        let p = parse_flexible(text, AxisOrder::LatLon).unwrap();
        near(p.lat, -37.673, 1e-8);
        near(p.lon, 144.843, 1e-8);
    }
    assert_eq!(
        parse_flexible("0,0", AxisOrder::LatLon).unwrap(),
        Coordinate { lat: 0.0, lon: 0.0 }
    );
}

#[test]
fn invalid_input_is_not_silently_repaired() {
    for text in [
        "91,0",
        "0,181",
        "N906001 E1800000",
        "N396000 E1162400",
        "N395460 E1162400",
        "N-39.9 E116.4",
        "39.9,116.4,1",
        "39 54 00 116",
        "39 54 00 116 24",
        "NaN,2",
        "Infinity,0",
        "N395400 Q1162400",
        "N39.5 20 E116 24",
        "--39.9,116.4",
        "N039.54.00.000junk E116.24.00.000",
    ] {
        assert!(parse_flexible(text, AxisOrder::LatLon).is_err(), "{text}");
    }
    assert_eq!(
        parse_flexible("N396000 E1162400", AxisOrder::LatLon),
        Err(FlexibleParseError::MinuteOrSecondOutOfRange)
    );
    assert_eq!(
        parse_flexible("91,0", AxisOrder::LatLon),
        Err(FlexibleParseError::LatitudeOutOfRange)
    );
    assert_eq!(
        FlexibleParseError::WrongHemisphere.to_string(),
        "纬度使用 N/S，经度使用 E/W"
    );
}

#[test]
fn accepts_everything_from_str_accepts() {
    for text in [
        "N400000E1160000",
        "N403036E1163036",
        "N400000.0E1160000.0",
        "N403003.6E1163003.6",
        "N040.00.00.000 E116.00.00.000",
        "N040.30.03.600 E116.30.03.600",
        "N 40 00.0 E 116 00.0",
        "N 40 30.6 E 116 30.6",
        "40.123456,116.654321",
        "+40.123456 +116.654321",
        "N4030.6E11630.6",
        "N39113625E116342500",
    ] {
        let expected: Coordinate = text.parse().unwrap();
        let p = parse_flexible(text, AxisOrder::LatLon).unwrap_or_else(|e| panic!("{text}: {e}"));
        near(p.lat, expected.lat, 1e-8);
        near(p.lon, expected.lon, 1e-8);
    }
}
