use lazy_static::lazy_static;
use navdata_components::algorithm::position_transformation::get_colocated_coord;
use navdata_components::coordinate::Coordinate;

lazy_static! {
    static ref RAW: Coordinate = Coordinate {
        lat: 40.0,
        lon: 116.0
    };
}

/// 1e-9 度 ≈ 0.1 毫米，远小于 ARINC 424 的 1/100 秒（约 0.3 米）。
///
/// `get_colocated_coord` 走 Vincenty 正解，末尾的 `atan2` / `sin` 在不同平台的
/// libm 上会差 1 ULP（约 1e-14 度）。拿 `assert_eq!` 比浮点会让测试随编译器和
/// 平台飘，所以这里用容差。
const TOLERANCE_DEGREES: f64 = 1e-9;

fn assert_close(actual: Coordinate, expected: Coordinate) {
    let lat_error = (actual.lat - expected.lat).abs();
    let lon_error = (actual.lon - expected.lon).abs();
    assert!(
        lat_error < TOLERANCE_DEGREES,
        "纬度偏差 {lat_error:e} 超过容差: 实际 {actual:?}, 期望 {expected:?}"
    );
    assert!(
        lon_error < TOLERANCE_DEGREES,
        "经度偏差 {lon_error:e} 超过容差: 实际 {actual:?}, 期望 {expected:?}"
    );
}

#[test]
fn test1() {
    assert_close(
        get_colocated_coord(&RAW, 112.0, 36502.0, 0.0),
        Coordinate {
            lat: 39.87617306290192,
            lon: 116.39561635066224,
        },
    );
}

#[test]
fn test2() {
    assert_close(
        get_colocated_coord(&RAW, 232.0, 2354.5, 0.0),
        Coordinate {
            lat: 39.986942788836465,
            lon: 115.97827691463395,
        },
    );
}

#[test]
fn test3() {
    assert_close(
        get_colocated_coord(&RAW, 232.0, 2354.5, 12.0),
        Coordinate {
            lat: 39.99070164156582,
            lon: 115.97522159435215,
        },
    );
}
