use navdata_components::algorithm::measurement::{get_azimuth, get_distance};
use navdata_components::algorithm::position_transformation::get_colocated_coord;
use navdata_components::algorithm::shape::ArcDirection;
use navdata_components::boundary::{
    boundary_path, parse_boundary, parse_boundary_path, BoundaryError, BoundaryPart,
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
    lat: 37.0 + 44.0 / 60.0 + 43.0 / 3600.0,
    lon: 112.0 + 36.0 / 60.0 + 37.0 / 3600.0,
};
const C: &str = "N374443E1123637";

/// 圆心按给定方位、距离外推后写成航图常用的整秒坐标。
fn at(bearing: f64, radius: f64) -> String {
    get_colocated_coord(&CENTER, bearing, radius, 0.0).format(CoordinateFormat::Dms)
}

const ENGLISH: &str = "N375514E1131326-N375917E1132752-N375707E1132929-N375208E1133238-N374431E1133003-N374434E1132200-a clockwise circle with a radius of 66.698km centered on N374443E1123637-N373819E1132115-N374004E1130916-a counterclockwise circle with a radius of 48.758km centered on N374443E1123637-N374236E1130941-N374212E1131534-a counterclockwise circle with a radius of 57.433km centered on N374443E1123637-N375514E1131326";
const CHINESE: &str =
    "N375514E1131326-N375917E1132752-N375707E1132929-N375208E1133238-N374431E1133003-N374434E113
2200-以 N374443E1123637 为圆心，半径 66.698km 顺时针的弧-N373819E1132115-N374004E1130916-以
N374443E1123637 为圆心，半径 48.758km 逆时针的弧-N374236E1130941-N374212E1131534-以
N374443E1123637 为圆心，半径 57.433km 逆时针的弧-N375514E1131326";

fn arc_of(part: &BoundaryPart) -> (f64, ArcDirection) {
    match part {
        BoundaryPart::Arc {
            center,
            radius,
            direction,
        } => {
            assert_eq!(*center, CENTER);
            (*radius, *direction)
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn chart_text_in_both_languages() {
    let english = parse_boundary(ENGLISH).unwrap();
    assert_eq!(english, parse_boundary(CHINESE).unwrap());
    assert_eq!(english.len(), 14);
    assert_eq!(arc_of(&english[6]), (66_698.0, ArcDirection::Clockwise));
    assert_eq!(
        arc_of(&english[9]),
        (48_758.0, ArcDirection::CounterClockwise)
    );
    assert_eq!(
        arc_of(&english[12]),
        (57_433.0, ArcDirection::CounterClockwise)
    );
    // 原文在 N374434E113⏎2200 处换行
    assert_eq!(
        english[5],
        BoundaryPart::Point(
            Coordinate::parse_flexible("N374434E1132200", Default::default()).unwrap()
        )
    );

    let path = boundary_path(&english, 1.0).unwrap();
    assert_eq!(path.points.first(), path.points.last());
    assert!(path.unresolved.is_empty() && path.warnings.is_empty());
    // 第一段圆弧的中间点都在 66.698 km 上，方位顺时针递增
    let start = path
        .points
        .iter()
        .position(|p| *p == english_point(5))
        .unwrap();
    let end = path
        .points
        .iter()
        .position(|p| *p == english_point(7))
        .unwrap();
    let mut previous = get_azimuth(&CENTER, &path.points[start]);
    for p in &path.points[start + 1..end] {
        near(get_distance(&CENTER, p), 66_698.0, 1e-3);
        let bearing = get_azimuth(&CENTER, p);
        assert!(bearing > previous, "{bearing} <= {previous}");
        previous = bearing;
    }
}

fn english_point(i: usize) -> Coordinate {
    match parse_boundary(ENGLISH).unwrap()[i] {
        BoundaryPart::Point(p) => p,
        ref other => panic!("{other:?}"),
    }
}

#[test]
fn phrasing_variants_fill_the_same_slots() {
    for (text, radius, direction) in [
        (
            format!("以 {C} 为圆心，半径 5.500km 顺时针的弧"),
            5_500.0,
            ArcDirection::Clockwise,
        ),
        (
            format!("以{C}为中心、半径3海里逆时针方向的弧线"),
            5_556.0,
            ArcDirection::CounterClockwise,
        ),
        (
            format!("{C} 为圆心 半径 2.5NM 的顺时针圆弧"),
            4_630.0,
            ArcDirection::Clockwise,
        ),
        (
            format!("圆心为{C}，半径为800米，反时针"),
            800.0,
            ArcDirection::CounterClockwise,
        ),
        (
            format!("顺时针沿以{C}为圆心、半径5.5千米的圆弧"),
            5_500.0,
            ArcDirection::Clockwise,
        ),
        (
            format!("thence clockwise along the arc of a circle radius 5 NM centred on {C}"),
            9_260.0,
            ArcDirection::Clockwise,
        ),
        (
            format!("an anti-clockwise arc of 3 NM radius centre {C}"),
            5_556.0,
            ArcDirection::CounterClockwise,
        ),
        (
            format!("counter-\nclockwise arc, R=12km, centered at {C}"),
            12_000.0,
            ArcDirection::CounterClockwise,
        ),
        (
            format!("arc (CW) radius 10 kilometres centre point {C}"),
            10_000.0,
            ArcDirection::Clockwise,
        ),
        (
            "a clockwise circle with a radius of 66.698km centered on N374443\nE1123637"
                .to_string(),
            66_698.0,
            ArcDirection::Clockwise,
        ),
    ] {
        let parts = parse_boundary(&text).unwrap_or_else(|e| panic!("{text}: {e}"));
        assert_eq!(parts.len(), 1, "{text}: {parts:?}");
        assert_eq!(arc_of(&parts[0]), (radius, direction), "{text}");
    }
    for text in [
        format!("以 {C} 为中心，半径 10 公里的圆"),
        format!("a circle with a radius of 10 km centred on {C}"),
        format!("circle radius 10KM centre {C}"),
        format!("以{C}为圆心半径10公里范围"),
    ] {
        match parse_boundary(&text).unwrap_or_else(|e| panic!("{text}: {e}"))[..] {
            [BoundaryPart::Circle { center, radius }] => {
                assert_eq!(center, CENTER);
                near(radius, 10_000.0, 1e-9);
            }
            ref other => panic!("{text}: {other:?}"),
        }
    }
}

#[test]
fn separators_and_filler_words() {
    let a = at(0.0, 5_500.0);
    let b = at(90.0, 5_500.0);
    let c = at(180.0, 9_000.0);
    for text in [
        format!("{a}－以{C}为圆心，半径5.5km顺时针的弧－{b}－{c}－回到起点"),
        format!("{a} 至 以{C}为圆心，半径5.5km顺时针的弧 至 {b} 至 {c} 至 起点"),
        format!("{a}, thence clockwise along the arc of a circle radius 5.5 km centred on {C} to {b}, then to {c} and back to the point of origin"),
        format!("From {a} a clockwise arc radius 5.5km centred at {C} to {b}; {c}; to the starting point"),
    ] {
        let parts = parse_boundary(&text).unwrap_or_else(|e| panic!("{text}: {e}"));
        assert_eq!(parts.len(), 4, "{text}: {parts:?}");
        let path = boundary_path(&parts, 1.0).unwrap();
        assert!(path.unresolved.is_empty(), "{text}: {:?}", path.unresolved);
    }
}

#[test]
fn arcs_follow_direction_and_wrap_around() {
    let text = format!(
        "{}至以{C}为圆心，半径5.5公里逆时针的弧至{}",
        at(0.0, 5_500.0),
        at(90.0, 5_500.0)
    );
    let path = parse_boundary_path(&text, 1.0).unwrap();
    // 逆时针从 0° 走到 90° 扫过约 270°
    assert!(
        (271..=274).contains(&path.points.len()),
        "{}",
        path.points.len()
    );
    let mid = get_azimuth(&CENTER, &path.points[135]);
    assert!((215.0..235.0).contains(&mid), "{mid}");

    // 首段圆弧从最后一个点出发
    let text = format!(
        "以{C}为圆心，半径5.5km顺时针的弧，{}，{}，{}",
        at(90.0, 5_500.0),
        at(180.0, 9_000.0),
        at(0.0, 5_500.0)
    );
    let path = parse_boundary_path(&text, 1.0).unwrap();
    assert_eq!(path.points.first(), path.points.last());
    assert!(
        (93..=95).contains(&path.points.len()),
        "{}",
        path.points.len()
    );

    // 末段圆弧回到第一个点
    let text = format!(
        "{}－{}－以{C}为圆心，半径5.5km顺时针的弧",
        at(90.0, 5_500.0),
        at(180.0, 9_000.0)
    );
    let path = parse_boundary_path(&text, 1.0).unwrap();
    assert_eq!(path.points.first(), path.points.last());
}

#[test]
fn whole_circle_and_point_lists() {
    let path = parse_boundary_path(&format!("以 {C} 为中心，半径 10 公里的圆"), 1.0).unwrap();
    assert_eq!(path.points.len(), 361);
    for p in &path.points {
        near(get_distance(&CENTER, p), 10_000.0, 1e-3);
    }
    let path =
        parse_boundary_path("N390000E1160000－N400000E1160000－N400000E1170000", 1.0).unwrap();
    assert_eq!(path.points.len(), 3);
}

#[test]
fn unresolved_text_becomes_a_marked_straight_edge() {
    let path = parse_boundary_path(
        "N390000E1160000－沿国境线－N400000E1160000－N400000E1170000",
        1.0,
    )
    .unwrap();
    assert_eq!(path.points.len(), 3);
    assert_eq!(path.unresolved.len(), 1);
    assert_eq!(
        (path.unresolved[0].from, path.unresolved[0].text.as_str()),
        (0, "沿国境线")
    );

    let path = parse_boundary_path(
        "N390000E1160000 - N400000E1160000 - N400000E1170000 - along the FIR boundary",
        1.0,
    )
    .unwrap();
    assert_eq!(path.points.len(), 4);
    assert_eq!(path.points[0], path.points[3]);
    assert_eq!(path.unresolved[0].from, 2);
    assert_eq!(path.unresolved[0].text, "ALONG THE FIR BOUNDARY");

    // 没写方向的圆弧也按直线连接并标出
    let text = format!(
        "{}－以{C}为圆心，半径5.5km的弧－{}",
        at(0.0, 5_500.0),
        at(90.0, 5_500.0)
    );
    let path = parse_boundary_path(&text, 1.0).unwrap();
    assert_eq!(path.points.len(), 2);
    assert!(path.unresolved[0].text.contains("圆心"));
}

#[test]
fn arc_endpoints_far_from_the_radius_are_reported() {
    let text = format!(
        "{}－以{C}为圆心，半径5.5km顺时针的弧－{}",
        at(0.0, 8_000.0),
        at(90.0, 5_500.0)
    );
    let path = parse_boundary_path(&text, 1.0).unwrap();
    assert_eq!(path.warnings.len(), 1);
    near(path.warnings[0].distance, 8_000.0, 31.0);
    let mut warning = path.warnings[0];
    warning.distance = 8_000.0;
    assert_eq!(
        warning.to_string(),
        "圆弧端点距圆心 8.000 km，半径为 5.500 km"
    );
}

#[test]
fn errors() {
    assert_eq!(parse_boundary("沿国境线"), Err(BoundaryError::Empty));
    assert!(matches!(
        parse_boundary(&format!("以{C}为圆心，半径0km的圆")),
        Err(BoundaryError::InvalidRadius(_))
    ));
    assert!(matches!(
        parse_boundary("N396000E1160000－N400000E1170000"),
        Err(BoundaryError::Coordinate { .. })
    ));
    let two_arcs = format!(
        "{}－以{C}为圆心，半径5.5km顺时针的弧－以{C}为圆心，半径5.5km顺时针的弧－{}",
        at(0.0, 5_500.0),
        at(90.0, 5_500.0)
    );
    assert_eq!(
        parse_boundary_path(&two_arcs, 1.0),
        Err(BoundaryError::ArcMissingEndpoint)
    );
    assert_eq!(
        parse_boundary_path(
            &format!("N390000E1160000－以{C}为中心，半径10公里的圆"),
            1.0
        ),
        Err(BoundaryError::CircleMixedWithPoints)
    );
}
