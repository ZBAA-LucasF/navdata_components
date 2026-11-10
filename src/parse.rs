//! 宽松的坐标文本解析。
//!
//! [`Coordinate::from_str`](std::str::FromStr) 只接受固定的几种写法，适合解析
//! 数据库与 ARINC 424 字段；这里的 [`parse_flexible`] 面向人工输入和航图正文：
//! 半球标记可写在开头或末尾、经纬度可前后颠倒、度分秒可用空格或 `°′″` 分隔，
//! 也接受紧凑写法（`N372857E1125818`）和十进制度。
//!
//! 解析是严格的：分秒越界、半球标记与正负号混用、多余字符等一律报错，不会
//! 悄悄「修正」输入。

use crate::coordinate::Coordinate;
use std::fmt::{Display, Formatter};

/// 没有半球标记时，两个数值的先后顺序。
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum AxisOrder {
    /// 先纬度后经度（默认）
    #[default]
    LatLon,
    /// 先经度后纬度（如 GeoJSON）
    LonLat,
}

/// 宽松解析坐标时的错误。`Display` 输出可直接展示给用户的中文说明。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlexibleParseError {
    /// 单行文本过长
    TooLong,
    /// 写了经度标记却找不到纬度
    MissingLatitude,
    /// 写了纬度标记却找不到经度
    MissingLongitude,
    /// 没有半球标记时，不能拆成恰好两个数值
    NotAPair,
    /// 半球标记出现多次，或不在数值的开头/末尾
    MisplacedHemisphere,
    /// 纬度用了 E/W，或经度用了 N/S
    WrongHemisphere,
    /// 半球标记与正负号同时出现
    HemisphereWithSign,
    /// 含有无法识别的字符
    UnrecognizedFormat,
    /// 度、分或秒不是合法的数字
    InvalidNumber,
    /// 紧凑写法（不含分隔符）缺少半球标记或度数位数不对
    AmbiguousCompact,
    /// 度分秒中只有最后一项可以带小数
    FractionNotLast,
    /// 分或秒不小于 60
    MinuteOrSecondOutOfRange,
    /// 纬度超出 ±90°
    LatitudeOutOfRange,
    /// 经度超出 ±180°
    LongitudeOutOfRange,
}

impl Display for FlexibleParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        use FlexibleParseError::*;
        f.write_str(match self {
            TooLong => "单行坐标过长",
            MissingLatitude => "缺少纬度",
            MissingLongitude => "缺少经度",
            NotAPair => "每行输入一对坐标，如 39.9, 116.4 或 N395400 E1162400",
            MisplacedHemisphere => "半球标记只能在坐标开头或末尾出现一次",
            WrongHemisphere => "纬度使用 N/S，经度使用 E/W",
            HemisphereWithSign => "半球标记与正负号不能同时使用",
            UnrecognizedFormat => "无法识别坐标格式",
            InvalidNumber => "无效的度分秒数值",
            AmbiguousCompact => "紧凑坐标需使用 N/S、E/W 标记及完整度数位",
            FractionNotLast => "只有度分秒的最后一项可以带小数",
            MinuteOrSecondOutOfRange => "分和秒必须小于 60",
            LatitudeOutOfRange => "纬度必须在 -90 到 90 之间",
            LongitudeOutOfRange => "经度必须在 -180 到 180 之间",
        })
    }
}

impl std::error::Error for FlexibleParseError {}

const DMS_SEPARATORS: &str = "°º'′\"″:";
const PAIR_SEPARATORS: [char; 3] = [',', ';', ' '];

fn number(text: &str) -> Result<f64, FlexibleParseError> {
    text.parse().map_err(|_| FlexibleParseError::InvalidNumber)
}

/// 解析单个经度或纬度，返回带符号的十进制度。
fn axis(raw: &str, latitude: bool) -> Result<f64, FlexibleParseError> {
    use FlexibleParseError::*;
    let raw = raw.trim();
    let is_direction = |c: char| matches!(c, 'N' | 'S' | 'E' | 'W');
    let directions: Vec<_> = raw.chars().filter(|c| is_direction(*c)).collect();
    let direction = directions.first().copied();
    if directions.len() > 1
        || (direction.is_some() && !raw.starts_with(is_direction) && !raw.ends_with(is_direction))
    {
        return Err(MisplacedHemisphere);
    }
    if direction.is_some_and(|c| !(if latitude { "NS" } else { "EW" }).contains(c)) {
        return Err(WrongHemisphere);
    }
    let value = raw.trim_matches(is_direction).trim();
    if direction.is_some() && value.starts_with(['+', '-']) {
        return Err(HemisphereWithSign);
    }
    let negative = direction.is_some_and(|c| "SW".contains(c)) || value.starts_with('-');
    let value = value.strip_prefix(['+', '-']).unwrap_or(value);
    if value.is_empty()
        || !value.chars().all(|c| {
            c.is_ascii_digit() || c == '.' || c == ' ' || c == '\t' || DMS_SEPARATORS.contains(c)
        })
    {
        return Err(UnrecognizedFormat);
    }
    let width = if latitude { 2 } else { 3 };
    let fields: Vec<_> = value.split('.').collect();
    let parts: Vec<f64> =
        if (3..=4).contains(&fields.len()) && fields[1].len() == 2 && fields[2].len() == 2 {
            // 扇区写法：DDD.MM.SS.sss
            vec![
                number(fields[0])?,
                number(fields[1])?,
                number(&fields[2..].join("."))?,
            ]
        } else if value
            .chars()
            .any(|c| c.is_whitespace() || DMS_SEPARATORS.contains(c))
        {
            value
                .split(|c: char| c.is_whitespace() || DMS_SEPARATORS.contains(c))
                .filter(|p| !p.is_empty())
                .map(number)
                .collect::<Result<_, _>>()?
        } else {
            let digits = fields[0].len();
            if digits <= 3 {
                vec![number(value)?]
            } else if direction.is_some() && digits == width + 2 {
                vec![number(&value[..width])?, number(&value[width..])?]
            } else if direction.is_some() && digits == width + 4 {
                vec![
                    number(&value[..width])?,
                    number(&value[width..width + 2])?,
                    number(&value[width + 2..])?,
                ]
            } else if direction.is_some() && digits == width + 6 && fields.len() == 1 {
                // 秒带两位小数但省略小数点：DDMMSSss
                vec![
                    number(&value[..width])?,
                    number(&value[width..width + 2])?,
                    number(&value[width + 2..])? / 100.0,
                ]
            } else {
                return Err(AmbiguousCompact);
            }
        };
    if parts.is_empty()
        || parts.len() > 3
        || parts.iter().any(|p| !p.is_finite() || *p < 0.0)
        || parts.iter().take(parts.len() - 1).any(|p| p.fract() != 0.0)
    {
        return Err(FractionNotLast);
    }
    if parts.get(1).is_some_and(|p| *p >= 60.0) || parts.get(2).is_some_and(|p| *p >= 60.0) {
        return Err(MinuteOrSecondOutOfRange);
    }
    let result =
        parts[0] + parts.get(1).unwrap_or(&0.0) / 60.0 + parts.get(2).unwrap_or(&0.0) / 3600.0;
    if result > if latitude { 90.0 } else { 180.0 } {
        return Err(if latitude {
            LatitudeOutOfRange
        } else {
            LongitudeOutOfRange
        });
    }
    Ok(if negative { -result } else { result })
}

/// 宽松地解析一对经纬度。
///
/// 支持的写法举例（均表示北纬 39.9°、东经 116.4°）：
///
/// - 十进制度：`39.9,116.4`、`39.9 116.4`、`+39.9 +116.4`
/// - 紧凑度分秒：`N395400E1162400`、`N395400.00 E1162400.00`、`N39540000 E116240000`
/// - 度分：`N3954.000 E11624.000`、`N 39 54.0 E 116 24.0`
/// - 扇区格式：`N039.54.00.000 E116.24.00.000`
/// - 符号：`N39°54′00″ E116°24′00″`、`39°54′00″N 116°24′00″E`
/// - 经度在前：`E1162400 N395400`、`116.4E 39.9N`
///
/// 没有半球标记时按 `order` 决定先后。
pub fn parse_flexible(text: &str, order: AxisOrder) -> Result<Coordinate, FlexibleParseError> {
    use FlexibleParseError::*;
    let text = text.trim().to_uppercase().replace('，', ",");
    if text.len() > 256 {
        return Err(TooLong);
    }
    let has_ns = text.contains(['N', 'S']);
    let has_ew = text.contains(['E', 'W']);
    let (lat, lon) = if has_ns && has_ew {
        if text.starts_with(['N', 'S']) {
            let at = text.find(['E', 'W']).ok_or(MissingLongitude)?;
            (
                axis(text[..at].trim_end_matches(PAIR_SEPARATORS), true)?,
                axis(&text[at..], false)?,
            )
        } else if text.starts_with(['E', 'W']) {
            let at = text.find(['N', 'S']).ok_or(MissingLatitude)?;
            (
                axis(&text[at..], true)?,
                axis(text[..at].trim_end_matches(PAIR_SEPARATORS), false)?,
            )
        } else {
            // 半球标记写在末尾
            let at = text.find(['N', 'S']).ok_or(MissingLatitude)? + 1;
            let (a, b) = text.split_at(at);
            if a.contains(['E', 'W']) {
                let at = text.find(['E', 'W']).ok_or(MissingLongitude)? + 1;
                (
                    axis(text[at..].trim_start_matches(PAIR_SEPARATORS), true)?,
                    axis(&text[..at], false)?,
                )
            } else {
                (
                    axis(a, true)?,
                    axis(b.trim_start_matches(PAIR_SEPARATORS), false)?,
                )
            }
        }
    } else {
        let parts: Vec<String> = if text.contains([',', ';']) {
            text.split([',', ';']).map(str::to_string).collect()
        } else {
            let words: Vec<_> = text.split_whitespace().collect();
            match words.len() {
                // 「度 分 [秒]」两两成组
                4 | 6 => words
                    .chunks(words.len() / 2)
                    .map(|half| half.join(" "))
                    .collect(),
                _ => words.into_iter().map(str::to_string).collect(),
            }
        };
        if parts.len() != 2 {
            return Err(NotAPair);
        }
        let (a, b) = match order {
            AxisOrder::LatLon => (&parts[0], &parts[1]),
            AxisOrder::LonLat => (&parts[1], &parts[0]),
        };
        (axis(a, true)?, axis(b, false)?)
    };
    Ok(Coordinate { lat, lon })
}

impl Coordinate {
    /// 见 [`parse_flexible`]。
    pub fn parse_flexible(text: &str, order: AxisOrder) -> Result<Self, FlexibleParseError> {
        parse_flexible(text, order)
    }
}
