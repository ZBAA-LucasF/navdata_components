use navdata_components::algorithm::measurement::get_distance;
use navdata_components::algorithm::shape::{arc, circle, sector, ArcDirection};
use navdata_components::coordinate::Coordinate;

const ORIGIN: Coordinate = Coordinate { lat: 0.0, lon: 0.0 };

fn near(actual: f64, expected: f64, tolerance: f64) {
    assert!(
        (actual - expected).abs() <= tolerance,
        "{actual} != {expected}"
    );
}

#[test]
fn arc_uses_true_bearings_and_direction() {
    let points = arc(&ORIGIN, 1000.0, 0.0, 90.0, ArcDirection::Clockwise, 1.0);
    assert_eq!(points.len(), 91);
    near(points[0].lon, 0.0, 1e-8);
    near(points[0].lat, 0.00904369477, 1e-8);
    near(points[90].lon, 0.00898315284, 1e-8);
    near(points[90].lat, 0.0, 1e-8);
    let center = Coordinate {
        lat: 39.0,
        lon: 116.0,
    };
    assert_eq!(
        arc(&center, 1e4, 350.0, 10.0, ArcDirection::Clockwise, 1.0).len(),
        21
    );
    assert_eq!(
        arc(
            &center,
            1e4,
            350.0,
            10.0,
            ArcDirection::CounterClockwise,
            1.0
        )
        .len(),
        341
    );
    assert_eq!(
        arc(&center, 1e4, 0.0, 90.0, ArcDirection::Clockwise, 5.0).len(),
        19
    );
    for p in arc(&center, 1e4, 350.0, 10.0, ArcDirection::Clockwise, 1.0) {
        near(get_distance(&center, &p), 1e4, 1e-3);
    }
}

#[test]
fn circle_and_sector_are_closed() {
    let ring = circle(&ORIGIN, 1852.0, 1.0);
    assert_eq!(ring.len(), 361);
    assert_eq!(ring[0], ring[360]);
    near(ring[0].lat, 0.0167489227101, 1e-8);
    let center = Coordinate {
        lat: 39.0,
        lon: 116.0,
    };
    let ring = sector(&center, 18520.0, 0.0, 90.0, ArcDirection::Clockwise, 1.0);
    assert_eq!(ring[91], center);
    assert_eq!(ring[0], ring[92]);
}

#[test]
fn longitudes_stay_in_range_near_the_antimeridian() {
    for p in circle(
        &Coordinate {
            lat: 80.0,
            lon: 179.9,
        },
        18520.0,
        1.0,
    ) {
        assert!((-180.0..180.0).contains(&p.lon), "{p:?}");
    }
}
