use navdata_components::Coordinate;

/// 解析回来的值必须和格式化前一致（1/100 秒的精度）。
fn round_trip(lat: f64, lon: f64) -> (String, String) {
    let coordinate = Coordinate { lat, lon };
    let lat_field = coordinate.to_arinc_lat();
    let lon_field = coordinate.to_arinc_lon();
    let parsed: Coordinate = format!("{lat_field}{lon_field}")
        .parse()
        .expect("生成的字段必须能被自己解析回来");
    assert!(
        (parsed.lat - lat).abs() < 1.0 / 360_000.0,
        "纬度往返失真: {lat} -> {lat_field} -> {}",
        parsed.lat
    );
    assert!(
        (parsed.lon - lon).abs() < 1.0 / 360_000.0,
        "经度往返失真: {lon} -> {lon_field} -> {}",
        parsed.lon
    );
    (lat_field, lon_field)
}

#[test]
fn writes_fixed_width_fields() {
    let (lat, lon) = round_trip(40.080_111, 116.584_444);
    assert_eq!(lat, "N40044840");
    assert_eq!(lon, "E116350400");
    assert_eq!(lat.len(), 9);
    assert_eq!(lon.len(), 10);
}

#[test]
fn writes_south_and_west() {
    let (lat, lon) = round_trip(-33.946_111, -70.794_444);
    assert_eq!(lat, "S33564600");
    assert_eq!(lon, "W070474000");
}

#[test]
fn equator_and_prime_meridian_are_positive() {
    // -0.0 不能写成 S/W，否则同一个点会变成两个。
    let (lat, lon) = round_trip(-0.0, -0.0);
    assert_eq!(lat, "N00000000");
    assert_eq!(lon, "E000000000");
}

#[test]
fn carries_seconds_into_minutes() {
    // 59.9999 秒四舍五入后是 60.00，必须进位到分而不是写出 60。
    let value = 1.0 / 60.0 - 1.0 / 360_000.0 / 2.0;
    let field = Coordinate {
        lat: value,
        lon: 0.0,
    }
    .to_arinc_lat();
    assert_eq!(field, "N00010000");
}

#[test]
fn covers_the_extremes() {
    let north = Coordinate {
        lat: 90.0,
        lon: 180.0,
    };
    assert_eq!(north.to_arinc_lat(), "N90000000");
    assert_eq!(north.to_arinc_lon(), "E180000000");

    let south = Coordinate {
        lat: -90.0,
        lon: -180.0,
    };
    assert_eq!(south.to_arinc_lat(), "S90000000");
    assert_eq!(south.to_arinc_lon(), "W180000000");
}
