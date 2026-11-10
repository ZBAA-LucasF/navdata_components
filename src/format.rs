//! 把坐标写成常见的航空文本格式。
//!
//! 所有格式都先把数值换成最小单位的整数再拆分，秒进位到分、分进位到度由整数
//! 运算完成（例如 39.999999999 写成 `N40000000`，不会出现 `N39595960`）。
//! 每种格式的输出都能被 [`parse_flexible`](crate::parse::parse_flexible) 读回。

use crate::coordinate::Coordinate;

/// 坐标的文本格式。示例均为南纬 37.673°、东经 144.843°。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoordinateFormat {
    /// `-37.67300000,144.84300000`
    Decimal,
    /// `-37.67300000 +144.84300000`
    SignedDecimal,
    /// 紧凑度分：`S3740.3800E14450.5800`
    DegreeMinute,
    /// 空格分隔的度分（PMDG 等使用）：`S 37 40.3800 E 144 50.5800`
    DegreeMinuteSpaced,
    /// 紧凑度分秒，秒取整：`S374023E1445035`
    Dms,
    /// 紧凑度分秒，秒带两位小数：`S374022.80E1445034.80`
    DmsDecimal,
    /// 扇区格式（EuroScope 等使用）：`S037.40.22.800 E144.50.34.800`
    Sector,
    /// ARINC 424 定长字段：`S37402280E144503480`
    Arinc,
    /// 带度分秒符号：`S37°40′22.80″ E144°50′34.80″`
    Symbol,
}

impl CoordinateFormat {
    /// 全部格式，顺序与枚举定义一致。
    pub const ALL: [CoordinateFormat; 9] = [
        CoordinateFormat::Decimal,
        CoordinateFormat::SignedDecimal,
        CoordinateFormat::DegreeMinute,
        CoordinateFormat::DegreeMinuteSpaced,
        CoordinateFormat::Dms,
        CoordinateFormat::DmsDecimal,
        CoordinateFormat::Sector,
        CoordinateFormat::Arinc,
        CoordinateFormat::Symbol,
    ];
}

fn hemisphere(value: f64, latitude: bool) -> char {
    match (latitude, value < 0.0) {
        (true, false) => 'N',
        (true, true) => 'S',
        (false, false) => 'E',
        (false, true) => 'W',
    }
}

/// 拆成 (整度, 度以下的余量, 余量的换算基数)。
fn scaled(value: f64, per_degree: u64, decimals: u32) -> (u64, u64, u64) {
    let unit = per_degree * 10u64.pow(decimals);
    let total = (value.abs() * unit as f64).round() as u64;
    (total / unit, total % unit, 10u64.pow(decimals))
}

fn decimal(value: f64, sign: bool) -> String {
    let mark = if value < 0.0 {
        "-"
    } else if sign {
        "+"
    } else {
        ""
    };
    format!("{mark}{:011.8}", value.abs())
}

fn dm(value: f64, latitude: bool, separator: &str) -> String {
    let width = if latitude { 2 } else { 3 };
    let (degrees, rest, scale) = scaled(value, 60, 4);
    format!(
        "{}{separator}{degrees:0width$}{separator}{:02}.{:04}",
        hemisphere(value, latitude),
        rest / scale,
        rest % scale
    )
}

fn dms(value: f64, latitude: bool, decimals: u32) -> (char, u64, u64, u64, u64) {
    let (degrees, rest, scale) = scaled(value, 3600, decimals);
    let seconds = rest / scale;
    (
        hemisphere(value, latitude),
        degrees,
        seconds / 60,
        seconds % 60,
        rest % scale,
    )
}

fn dms_compact(value: f64, latitude: bool, decimals: u32) -> String {
    let width = if latitude { 2 } else { 3 };
    let (h, d, m, s, f) = dms(value, latitude, decimals);
    if decimals == 0 {
        format!("{h}{d:0width$}{m:02}{s:02}")
    } else {
        format!("{h}{d:0width$}{m:02}{s:02}.{f:0w$}", w = decimals as usize)
    }
}

fn sector(value: f64, latitude: bool) -> String {
    let (h, d, m, s, f) = dms(value, latitude, 3);
    format!("{h}{d:03}.{m:02}.{s:02}.{f:03}")
}

fn symbol(value: f64, latitude: bool) -> String {
    let width = if latitude { 2 } else { 3 };
    let (h, d, m, s, f) = dms(value, latitude, 2);
    format!("{h}{d:0width$}°{m:02}′{s:02}.{f:02}″")
}

impl Coordinate {
    /// 按指定格式写出经纬度。
    pub fn format(&self, format: CoordinateFormat) -> String {
        let (lat, lon) = (self.lat, self.lon);
        match format {
            CoordinateFormat::Decimal => format!("{},{}", decimal(lat, false), decimal(lon, false)),
            CoordinateFormat::SignedDecimal => {
                format!("{} {}", decimal(lat, true), decimal(lon, true))
            }
            CoordinateFormat::DegreeMinute => {
                format!("{}{}", dm(lat, true, ""), dm(lon, false, ""))
            }
            CoordinateFormat::DegreeMinuteSpaced => {
                format!("{} {}", dm(lat, true, " "), dm(lon, false, " "))
            }
            CoordinateFormat::Dms => {
                format!(
                    "{}{}",
                    dms_compact(lat, true, 0),
                    dms_compact(lon, false, 0)
                )
            }
            CoordinateFormat::DmsDecimal => {
                format!(
                    "{}{}",
                    dms_compact(lat, true, 2),
                    dms_compact(lon, false, 2)
                )
            }
            CoordinateFormat::Sector => format!("{} {}", sector(lat, true), sector(lon, false)),
            CoordinateFormat::Arinc => format!("{}{}", self.to_arinc_lat(), self.to_arinc_lon()),
            CoordinateFormat::Symbol => format!("{} {}", symbol(lat, true), symbol(lon, false)),
        }
    }
}
