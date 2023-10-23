use lazy_static::lazy_static;
use navdata_components::coordinate::Coordinate;

lazy_static! {
    static ref C1: Coordinate = Coordinate {
        lat: 40.0,
        lon: 116.0
    };
    static ref C2: Coordinate = Coordinate {
        lat: 40.5,
        lon: 116.0
    };
    static ref C3: Coordinate = Coordinate {
        lat: 40.5,
        lon: 116.5
    };
    static ref C4: Coordinate = Coordinate {
        lat: 40.501,
        lon: 116.0
    };
    static ref C5: Coordinate = Coordinate {
        lat: 40.501,
        lon: 116.501
    };
}

#[test]
fn degree() {
    assert_eq!(C1.lat_d(), 40);
    assert_eq!(C2.lat_d(), 40);
    assert_eq!(C3.lat_d(), 40);
    assert_eq!(C4.lat_d(), 40);
    assert_eq!(C5.lat_d(), 40);

    assert_eq!(C1.lon_d(), 116);
    assert_eq!(C2.lon_d(), 116);
    assert_eq!(C3.lon_d(), 116);
    assert_eq!(C4.lon_d(), 116);
    assert_eq!(C5.lon_d(), 116);
}

#[test]
fn minute() {
    assert_eq!(C1.lat_m(), 0);
    assert_eq!(C2.lat_m(), 30);
    assert_eq!(C3.lat_m(), 30);
    assert_eq!(C4.lat_m(), 30);
    assert_eq!(C5.lat_m(), 30);

    assert_eq!(C1.lon_m(), 0);
    assert_eq!(C2.lon_m(), 0);
    assert_eq!(C3.lon_m(), 30);
    assert_eq!(C4.lon_m(), 0);
    assert_eq!(C5.lon_m(), 30);
}

#[test]
fn second() {
    assert_eq!(C1.lat_s(), 0.0);
    assert_eq!(C2.lat_s(), 0.0);
    assert_eq!(C3.lat_s(), 0.0);
    assert_eq!(C4.lat_s(), 4.0);
    assert_eq!(C5.lat_s(), 4.0);

    assert_eq!(C1.lon_s(), 0.0);
    assert_eq!(C2.lon_s(), 0.0);
    assert_eq!(C3.lon_s(), 0.0);
    assert_eq!(C4.lon_s(), 0.0);
    assert_eq!(C5.lon_s(), 4.0);
}

#[test]
fn second_p() {
    assert_eq!((C1.lat_sp() * 10.0).round() / 10.0, 0.0);
    assert_eq!((C2.lat_sp() * 10.0).round() / 10.0, 0.0);
    assert_eq!((C3.lat_sp() * 10.0).round() / 10.0, 0.0);
    assert_eq!((C4.lat_sp() * 10.0).round() / 10.0, 3.6);
    assert_eq!((C5.lat_sp() * 10.0).round() / 10.0, 3.6);

    assert_eq!((C1.lon_sp() * 10.0).round() / 10.0, 0.0);
    assert_eq!((C2.lon_sp() * 10.0).round() / 10.0, 0.0);
    assert_eq!((C3.lon_sp() * 10.0).round() / 10.0, 0.0);
    assert_eq!((C4.lon_sp() * 10.0).round() / 10.0, 0.0);
    assert_eq!((C5.lon_sp() * 10.0).round() / 10.0, 3.6);
}

#[test]
fn minute_second() {
    assert_eq!((C1.lat_md() * 100.0).round() / 100.0, 0.0);
    assert_eq!((C2.lat_md() * 100.0).round() / 100.0, 30.0);
    assert_eq!((C3.lat_md() * 100.0).round() / 100.0, 30.0);
    assert_eq!((C4.lat_md() * 100.0).round() / 100.0, 30.06);
    assert_eq!((C5.lat_md() * 100.0).round() / 100.0, 30.06);

    assert_eq!((C1.lon_md() * 100.0).round() / 100.0, 0.0);
    assert_eq!((C2.lon_md() * 100.0).round() / 100.0, 0.0);
    assert_eq!((C3.lon_md() * 100.0).round() / 100.0, 30.0);
    assert_eq!((C4.lon_md() * 100.0).round() / 100.0, 0.0);
    assert_eq!((C5.lon_md() * 100.0).round() / 100.0, 30.06);
}

