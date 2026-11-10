use navdata_components::coordinate::Coordinate;
use navdata_components::format::CoordinateFormat::{self, *};
use navdata_components::parse::{parse_flexible, AxisOrder};

fn near(actual: f64, expected: f64, tolerance: f64) {
    assert!(
        (actual - expected).abs() <= tolerance,
        "{actual} != {expected}"
    );
}

#[test]
fn every_format_round_trips() {
    for input in ["-37.673,144.843", "5.25,-0.5", "0,0", "-89.999,179.999"] {
        let expected = parse_flexible(input, AxisOrder::LatLon).unwrap();
        for (format, tolerance) in CoordinateFormat::ALL
            .into_iter()
            .zip([1e-8, 1e-8, 1e-6, 1e-6, 3e-4, 3e-6, 3e-7, 3e-6, 3e-6])
        {
            let text = expected.format(format);
            let p =
                parse_flexible(&text, AxisOrder::LatLon).unwrap_or_else(|e| panic!("{text}: {e}"));
            near(p.lat, expected.lat, tolerance);
            near(p.lon, expected.lon, tolerance);
            if format != Symbol {
                let shared: Coordinate = text.parse().unwrap_or_else(|e| panic!("{text}: {e}"));
                near(shared.lat, expected.lat, tolerance);
                near(shared.lon, expected.lon, tolerance);
            }
        }
    }
}

#[test]
fn exact_output_and_carry() {
    let p = Coordinate {
        lat: -37.673,
        lon: 144.843,
    };
    for (format, expected) in [
        (Decimal, "-37.67300000,144.84300000"),
        (SignedDecimal, "-37.67300000 +144.84300000"),
        (DegreeMinute, "S3740.3800E14450.5800"),
        (DegreeMinuteSpaced, "S 37 40.3800 E 144 50.5800"),
        (Dms, "S374023E1445035"),
        (DmsDecimal, "S374022.80E1445034.80"),
        (Sector, "S037.40.22.800 E144.50.34.800"),
        (Arinc, "S37402280E144503480"),
        (Symbol, "S37°40′22.80″ E144°50′34.80″"),
    ] {
        assert_eq!(p.format(format), expected);
    }
    let p = Coordinate {
        lat: 39.999999999,
        lon: 116.999999999,
    };
    assert_eq!(p.format(Arinc), "N40000000E117000000");
    assert_eq!(p.format(Sector), "N040.00.00.000 E117.00.00.000");
    assert_eq!(p.format(DegreeMinute), "N4000.0000E11700.0000");
    assert!(p.format(Symbol).starts_with("N40°00′00.00″"));
}
