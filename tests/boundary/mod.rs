use navdata_components::algorithm::measurement::{get_azimuth, get_distance};
use navdata_components::algorithm::position_transformation::get_colocated_coord;
use navdata_components::algorithm::shape::ArcDirection;
use navdata_components::boundary::{
    boundary_ring, parse_boundary, parse_boundary_ring, BoundaryError, BoundaryPart,
};
use navdata_components::coordinate::Coordinate;
use navdata_components::format::CoordinateFormat;

fn near(actual: f64, expected: f64, tolerance: f64) {
    assert!(
        (actual - expected).abs() <= tolerance,
        "{actual} != {expected}"
    );
}

const CENTER: Coordinate = Coordinate {
    lat: 37.0 + 28.0 / 60.0 + 57.0 / 3600.0,
    lon: 112.0 + 58.0 / 60.0 + 18.0 / 3600.0,
};

/// 圆心按给定方位、距离外推后写成航图常用的整秒坐标。
fn on_circle(bearing: f64, radius: f64) -> String {
    get_colocated_coord(&CENTER, bearing, radius, 0.0).format(CoordinateFormat::Dms)
}

#[test]
fn parses_the_chart_arc_clause() {
    let parts = parse_boundary("以 N372857E1125818 为圆心，半径 5.500km 顺时针的弧").unwrap();
    let [BoundaryPart::Arc {
        center,
        radius,
        direction,
    }] = parts[..]
    else {
        panic!("{parts:?}");
    };
    near(center.lat, CENTER.lat, 1e-9);
    near(center.lon, CENTER.lon, 1e-9);
    near(radius, 5500.0, 1e-9);
    assert_eq!(direction, ArcDirection::Clockwise);

    for (text, metres, direction) in [
        (
            "以N372857E1125818为中心、半径3海里逆时针方向的弧线",
            5556.0,
            ArcDirection::CounterClockwise,
        ),
        (
            "N372857E1125818 为圆心 半径 2.5NM 的顺时针圆弧",
            4630.0,
            ArcDirection::Clockwise,
        ),
        (
            "以N37°28′57″E112°58′18″为圆心，半径为800米逆时针的弧",
            800.0,
            ArcDirection::CounterClockwise,
        ),
        (
            "以 n372857e1125818 为圆心，半径 5.5 千米 顺时针 弧",
            5500.0,
            ArcDirection::Clockwise,
        ),
    ] {
        match parse_boundary(text).unwrap_or_else(|e| panic!("{text}: {e}"))[..] {
            [BoundaryPart::Arc {
                radius,
                direction: d,
                ..
            }] => {
                near(radius, metres, 1e-9);
                assert_eq!(d, direction, "{text}");
            }
            ref other => panic!("{text}: {other:?}"),
        }
    }
}

#[test]
fn expands_points_and_arcs_into_a_closed_ring() {
    let radius = 5500.0;
    let text = format!(
        "{}－{}－以 N372857E1125818 为圆心，半径 5.500km 顺时针的弧－{}－{}－回到起点",
        on_circle(270.0, 12000.0),
        on_circle(0.0, radius),
        on_circle(90.0, radius),
        on_circle(180.0, 12000.0),
    );
    let ring = parse_boundary_ring(&text, 1.0).unwrap();
    assert_eq!(ring.first(), ring.last());
    // 2 个点 + 约 89 个弧上中间点 + 2 个点 + 闭合点
    let arc_points = ring.len() - 5;
    assert!((89..=91).contains(&arc_points), "{}", ring.len());
    let mut previous = 0.0;
    for p in &ring[2..2 + arc_points] {
        near(get_distance(&CENTER, p), radius, 1e-3);
        let bearing = get_azimuth(&CENTER, p);
        assert!(bearing > previous && bearing < 90.0, "{bearing}");
        previous = bearing;
    }
}

#[test]
fn counter_clockwise_takes_the_other_way_round() {
    let text = format!(
        "{}至以N372857E1125818为圆心，半径5.5公里逆时针的弧至{}",
        on_circle(0.0, 5500.0),
        on_circle(90.0, 5500.0),
    );
    let ring = parse_boundary_ring(&text, 1.0).unwrap();
    // 逆时针从 0° 走到 90° 要扫过约 270°（端点取整到秒，允许多一两个顶点）
    assert!(
        (1 + 269 + 1 + 1..=1 + 271 + 1 + 1).contains(&ring.len()),
        "{}",
        ring.len()
    );
    let mid = get_azimuth(&CENTER, &ring[135]);
    assert!((215.0..235.0).contains(&mid), "{mid}");
}

#[test]
fn a_leading_arc_starts_from_the_last_point() {
    let text = format!(
        "以N372857E1125818为圆心，半径5.5km顺时针的弧，{}，{}，{}",
        on_circle(90.0, 5500.0),
        on_circle(180.0, 9000.0),
        on_circle(0.0, 5500.0),
    );
    let ring = parse_boundary_ring(&text, 1.0).unwrap();
    assert_eq!(ring.first(), ring.last());
    near(get_distance(&CENTER, &ring[0]), 5500.0, 1e-3);
    assert!(
        (89 + 3 + 1..=91 + 3 + 1).contains(&ring.len()),
        "{}",
        ring.len()
    );
}

#[test]
fn whole_circle() {
    let ring = parse_boundary_ring("以 N390520E1162740 为中心，半径 10 公里的圆", 1.0).unwrap();
    assert_eq!(ring.len(), 361);
    assert_eq!(ring[0], ring[360]);
    let center = Coordinate::parse_flexible(
        "N390520E1162740",
        navdata_components::parse::AxisOrder::LatLon,
    )
    .unwrap();
    for p in &ring {
        near(get_distance(&center, p), 10_000.0, 1e-3);
    }
}

#[test]
fn plain_point_lists() {
    let ring =
        parse_boundary_ring("N390000E1160000－N400000E1160000－N400000E1170000", 1.0).unwrap();
    assert_eq!(ring.len(), 4);
    assert_eq!(ring[0], ring[3]);
}

#[test]
fn rejects_text_it_cannot_draw_faithfully() {
    assert_eq!(
        parse_boundary("N390000E1160000－沿国境线－N400000E1170000"),
        Err(BoundaryError::UnrecognizedText("－沿国境线－".into()))
    );
    assert!(matches!(
        parse_boundary("以N372857E1125818为圆心，半径5.5km的弧"),
        Err(BoundaryError::ArcWithoutDirection(_))
    ));
    assert!(matches!(
        parse_boundary("以N372857E1125818为圆心，半径0km的圆"),
        Err(BoundaryError::InvalidRadius(_))
    ));
    assert!(matches!(
        parse_boundary("N396000E1160000－N400000E1170000"),
        Err(BoundaryError::Coordinate { .. })
    ));
    assert_eq!(parse_boundary("　－　"), Err(BoundaryError::Empty));

    let off = format!(
        "{}－以N372857E1125818为圆心，半径5.5km顺时针的弧－{}",
        on_circle(0.0, 8000.0),
        on_circle(90.0, 5500.0)
    );
    assert!(matches!(
        parse_boundary_ring(&off, 1.0),
        Err(BoundaryError::ArcEndpointOffRadius { .. })
    ));
    let two_arcs = format!(
        "{}－以N372857E1125818为圆心，半径5.5km顺时针的弧－以N372857E1125818为圆心，半径5.5km顺时针的弧－{}",
        on_circle(0.0, 5500.0),
        on_circle(90.0, 5500.0)
    );
    assert_eq!(
        parse_boundary_ring(&two_arcs, 1.0),
        Err(BoundaryError::ArcMissingEndpoint)
    );
    assert_eq!(
        parse_boundary_ring(
            "N390000E1160000－以N390520E1162740为中心，半径10公里的圆",
            1.0
        ),
        Err(BoundaryError::CircleMixedWithPoints)
    );
    assert_eq!(
        boundary_ring(
            &parse_boundary("N390000E1160000－N400000E1170000").unwrap(),
            1.0
        ),
        Err(BoundaryError::TooFewPoints)
    );
    assert_eq!(
        BoundaryError::UnrecognizedText("沿国境线".into()).to_string(),
        "无法识别「沿国境线」"
    );
}
