//! 解析航图/AIP 正文中的空域边界描述（中文或英文）。
//!
//! ```text
//! N375514E1131326-N375917E1132752-以 N374443E1123637 为圆心，半径 66.698km 顺时针的弧-N373819E1132115
//! N375514E1131326-a clockwise circle with a radius of 66.698km centered on N374443E1123637-N373819E1132115
//! thence clockwise along the arc of a circle radius 5 NM centred on N374443E1123637 to N373819E1132115
//! 以 N390520E1162740 为中心，半径 10 公里的圆
//! ```
//!
//! 解析以带半球标记的坐标为锚点。紧挨着圆心关键词（`以…`、`…为圆心`、`圆心为…`、
//! `centered on …`、`centre at …` 等）的坐标是圆心，它前后的描述文字按槽位识别，
//! 各槽位的先后顺序任意：
//!
//! - 半径：`半径 5.5km`、`radius of 5.5 km`、`R=3NM`、`5 NM radius`；单位为
//!   km/kilometres/公里/千米、NM/nautical miles/海里、m/metres/米；
//! - 方向：顺时针/clockwise/CW，逆时针/反时针/counterclockwise/anticlockwise/CCW；
//! - 带方向的描述是圆弧，起点和终点取前后相邻的坐标点（首段圆弧从最后一个点出发，
//!   末段圆弧回到第一个点）；只有圆心和半径的描述是整圆。
//!
//! 片段之间用 `- － — ~ ; 至 到 to then thence` 分隔，`起点`、`point of origin` 等收尾词
//! 会被略过。其余文字（例如「沿国境线」「along the FIR boundary」）作为
//! [`BoundaryPart::Unresolved`] 保留，展开时用直线连接其前后两点，并在
//! [`BoundaryPath::unresolved`] 中标出这一段。
//!
//! 从 PDF 复制的文本常在坐标中间换行（`N374434E113⏎2200`）：两个数字之间的换行直接
//! 删除，其余换行视为空格。

use crate::algorithm::measurement::{get_azimuth, get_distance};
use crate::algorithm::shape::{arc, circle, ArcDirection};
use crate::coordinate::Coordinate;
use crate::parse::{parse_flexible, AxisOrder, FlexibleParseError};
use lazy_static::lazy_static;
use regex::Regex;
use std::fmt::{Display, Formatter};

/// 边界描述中的一个片段。半径单位为米。
#[derive(Debug, Clone, PartialEq)]
pub enum BoundaryPart {
    /// 边界上的一个点
    Point(Coordinate),
    /// 从前一个点沿圆弧到后一个点
    Arc {
        center: Coordinate,
        radius: f64,
        direction: ArcDirection,
    },
    /// 整圆
    Circle { center: Coordinate, radius: f64 },
    /// 无法换算成几何的原文，展开时用直线连接前后两点
    Unresolved(String),
}

/// 解析或展开边界描述时的错误。`Display` 输出中文说明。
#[derive(Debug, Clone, PartialEq)]
pub enum BoundaryError {
    /// 文本中缺少带半球标记的坐标
    Empty,
    /// 坐标写法有误
    Coordinate {
        text: String,
        error: FlexibleParseError,
    },
    /// 半径为 0
    InvalidRadius(String),
    /// 圆弧前后缺少坐标点（例如两段圆弧相连）
    ArcMissingEndpoint,
    /// 整圆与其他片段混在一起
    CircleMixedWithPoints,
}

impl Display for BoundaryError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            BoundaryError::Empty => write!(f, "没有找到带 N/S、E/W 标记的坐标"),
            BoundaryError::Coordinate { text, error } => write!(f, "坐标 {text}：{error}"),
            BoundaryError::InvalidRadius(text) => write!(f, "半径「{text}」必须大于 0"),
            BoundaryError::ArcMissingEndpoint => write!(f, "圆弧前后都需要有坐标点"),
            BoundaryError::CircleMixedWithPoints => {
                write!(f, "整圆需单独成段，与其他边界点之间用空行分开")
            }
        }
    }
}

impl std::error::Error for BoundaryError {}

/// 展开后的边界。
#[derive(Debug, Clone, PartialEq)]
pub struct BoundaryPath {
    /// 按原文顺序排列的顶点。描述以圆弧或未解析段收尾时，末尾补上第一个点。
    pub points: Vec<Coordinate>,
    /// 由未解析文字连接的边：`points[from]` 到 `points[from + 1]`
    pub unresolved: Vec<UnresolvedEdge>,
    /// 圆弧端点到圆心的距离与半径相差超过 max(10%, 500 m) 的情况
    pub warnings: Vec<ArcWarning>,
}

/// 用直线代替的一段边界。
#[derive(Debug, Clone, PartialEq)]
pub struct UnresolvedEdge {
    pub from: usize,
    pub text: String,
}

/// 圆弧端点偏离半径。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ArcWarning {
    pub point: Coordinate,
    pub radius: f64,
    pub distance: f64,
}

impl Display for ArcWarning {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "圆弧端点距圆心 {:.3} km，半径为 {:.3} km",
            self.distance / 1000.0,
            self.radius / 1000.0
        )
    }
}

/// 带半球标记的坐标，纬度在前或经度在前都可以。
const COORD: &str = r#"(?:[NS]\s*\d[\d°º'′"″.:\s]*?[,，]?\s*[EW]\s*\d[\d°º'′"″.:]*(?:\s+\d[\d°º'′"″.:]*){0,2}|[EW]\s*\d[\d°º'′"″.:\s]*?[,，]?\s*[NS]\s*\d[\d°º'′"″.:]*(?:\s+\d[\d°º'′"″.:]*){0,2})"#;
const UNIT: &str =
    r"(KILOMET(?:ER|RE)S?|KM|NAUTICAL\s+MILES?|NM|MET(?:ER|RE)S?|公里|千米|海里|米|M)(?:[^A-Z]|$)";

lazy_static! {
    static ref RE_COORD: Regex = Regex::new(COORD).unwrap();
    static ref RE_DIGIT_BREAK: Regex = Regex::new(r"(\d)[ \t]*\r?\n[ \t]*(\d)").unwrap();
    static ref RE_BREAK: Regex = Regex::new(r"\s*\r?\n\s*").unwrap();
    static ref RE_COUNTER_SPLIT: Regex =
        Regex::new(r"(COUNTER|ANTI)\s*-?\s*CLOCK\s*-?\s*WISE").unwrap();
    static ref RE_CLOCK_SPLIT: Regex = Regex::new(r"CLOCK\s*-\s*WISE").unwrap();
    static ref RE_SEPARATOR: Regex =
        Regex::new(r"[-－—–~～;；]|至|到|\bTO\b|\bTHEN(?:CE)?\b").unwrap();
    static ref RE_CENTER_BEFORE: Regex = Regex::new(
        r"(?:\bCENT(?:ER|RE)(?:ED|D|\s+POINT)?(?:\s+(?:ON|AT|OF|IS))?|(?:圆心|中心)(?:为|是|在)?|以)\s*[:：]?\s*$"
    )
    .unwrap();
    static ref RE_CENTER_AFTER: Regex =
        Regex::new(r"^\s*(?:(?:为|作为|是)?\s*(?:圆心|中心)|AS\s+(?:THE\s+)?CENT(?:ER|RE)\b)").unwrap();
    static ref RE_RADIUS: Regex = Regex::new(&format!(
        r"(?:RADIUS|半径|\bR\b)\s*(?:OF|IS|=|为|是|:|：)?\s*(\d+(?:\.\d+)?)\s*{UNIT}"
    ))
    .unwrap();
    static ref RE_RADIUS_AFTER: Regex =
        Regex::new(&format!(r"(\d+(?:\.\d+)?)\s*{UNIT}\s*(?:的)?\s*(?:RADIUS|半径)")).unwrap();
    static ref RE_COUNTER: Regex =
        Regex::new(r"COUNTERCLOCKWISE|ANTICLOCKWISE|\bCCW\b|逆时针|反时针").unwrap();
    static ref RE_CLOCKWISE: Regex = Regex::new(r"CLOCKWISE|\bCW\b|顺时针").unwrap();
    static ref RE_ARC: Regex = Regex::new(r"ARC|弧").unwrap();
    static ref RE_FILLER: Regex = Regex::new(
        r"[\s,，、.。:：()（）]+|起点|终点|回|返回|闭合|然后|再|经|从|自|直线|连线|\b(?:BACK|POINT|OF|ORIGIN|BEGINNING|START(?:ING)?|FROM|THE|A|AN|AT|AND|STRAIGHT|LINE|VIA)\b"
    )
    .unwrap();
}

/// 统一大小写、连字符和换行，便于后续匹配。
fn normalize(text: &str) -> String {
    let text = text.to_uppercase();
    let text = RE_DIGIT_BREAK.replace_all(&text, "$1$2");
    let text = RE_BREAK.replace_all(&text, " ");
    let text = RE_COUNTER_SPLIT.replace_all(&text, "${1}CLOCKWISE");
    RE_CLOCK_SPLIT.replace_all(&text, "CLOCKWISE").into_owned()
}

/// 按分隔符拆开坐标之间的文字。
fn pieces(gap: &str) -> Vec<&str> {
    let mut result = Vec::new();
    let mut start = 0;
    for m in RE_SEPARATOR.find_iter(gap) {
        result.push(&gap[start..m.start()]);
        start = m.end();
    }
    result.push(&gap[start..]);
    result
}

/// 坐标之间除分隔符和收尾词外的文字记为未解析片段。
fn push_unresolved(piece: &str, parts: &mut Vec<BoundaryPart>) {
    if !RE_FILLER.replace_all(piece, "").trim().is_empty() {
        parts.push(BoundaryPart::Unresolved(piece.trim().to_string()));
    }
}

fn coordinate(text: &str) -> Result<Coordinate, BoundaryError> {
    parse_flexible(text, AxisOrder::LatLon).map_err(|error| BoundaryError::Coordinate {
        text: text.trim().to_string(),
        error,
    })
}

fn radius(description: &str) -> Option<(f64, String)> {
    let captures = RE_RADIUS
        .captures(description)
        .or_else(|| RE_RADIUS_AFTER.captures(description))?;
    let value: f64 = captures[1].parse().ok()?;
    let unit = &captures[2];
    let metres = value
        * if unit.starts_with('K') || unit == "公里" || unit == "千米" {
            1000.0
        } else if unit.starts_with('N') || unit == "海里" {
            1852.0
        } else {
            1.0
        };
    Some((metres, format!("{}{unit}", &captures[1])))
}

/// 把圆心坐标及其前后的描述文字换算成圆弧或整圆。
fn radial(
    center: Coordinate,
    description: &str,
    original: String,
) -> Result<BoundaryPart, BoundaryError> {
    let Some((metres, text)) = radius(description) else {
        return Ok(BoundaryPart::Unresolved(original));
    };
    if !(metres.is_finite() && metres > 0.0) {
        return Err(BoundaryError::InvalidRadius(text));
    }
    let direction = if RE_COUNTER.is_match(description) {
        Some(ArcDirection::CounterClockwise)
    } else if RE_CLOCKWISE.is_match(description) {
        Some(ArcDirection::Clockwise)
    } else {
        None
    };
    Ok(match direction {
        Some(direction) => BoundaryPart::Arc {
            center,
            radius: metres,
            direction,
        },
        None if RE_ARC.is_match(description) => BoundaryPart::Unresolved(original),
        None => BoundaryPart::Circle {
            center,
            radius: metres,
        },
    })
}

/// 把边界描述拆成片段。几何展开见 [`boundary_path`]。
pub fn parse_boundary(text: &str) -> Result<Vec<BoundaryPart>, BoundaryError> {
    let text = normalize(text);
    let matches: Vec<_> = RE_COORD.find_iter(&text).collect();
    if matches.is_empty() {
        return Err(BoundaryError::Empty);
    }
    let mut gaps = Vec::with_capacity(matches.len() + 1);
    let mut position = 0;
    for m in &matches {
        gaps.push(pieces(&text[position..m.start()]));
        position = m.end();
    }
    gaps.push(pieces(&text[position..]));

    let is_center: Vec<bool> = (0..matches.len())
        .map(|i| {
            RE_CENTER_BEFORE.is_match(gaps[i].last().unwrap())
                || RE_CENTER_AFTER.is_match(gaps[i + 1][0])
        })
        .collect();

    let mut parts = Vec::new();
    for (i, m) in matches.iter().enumerate() {
        let gap = &gaps[i];
        // 前一个圆心用掉本段开头，本坐标若是圆心则用掉本段结尾。
        let skip_first = usize::from(i > 0 && is_center[i - 1]);
        let skip_last = usize::from(is_center[i]);
        let end = gap.len().saturating_sub(skip_last).max(skip_first);
        for piece in &gap[skip_first..end] {
            push_unresolved(piece, &mut parts);
        }
        let point = coordinate(m.as_str())?;
        if is_center[i] {
            let before = gap.last().unwrap();
            let after = gaps[i + 1][0];
            let description = format!("{before} {after}");
            let original = format!("{}{}{}", before.trim_start(), m.as_str(), after.trim_end());
            parts.push(radial(point, &description, original)?);
        } else {
            parts.push(BoundaryPart::Point(point));
        }
    }
    let tail = &gaps[matches.len()];
    let skip_first = usize::from(is_center[matches.len() - 1]);
    for piece in &tail[skip_first.min(tail.len())..] {
        push_unresolved(piece, &mut parts);
    }
    Ok(parts)
}

fn push_distinct(points: &mut Vec<Coordinate>, point: Coordinate) {
    if points.last() != Some(&point) {
        points.push(point);
    }
}

/// 把片段展开成顶点序列。
///
/// 圆弧按 WGS-84 正解生成，相邻顶点的圆心角不超过 `max_step` 度，两端直接使用原文
/// 给出的坐标点。整圆单独成段时返回闭合的圆。
pub fn boundary_path(parts: &[BoundaryPart], max_step: f64) -> Result<BoundaryPath, BoundaryError> {
    if parts
        .iter()
        .any(|p| matches!(p, BoundaryPart::Circle { .. }))
    {
        return match parts {
            [BoundaryPart::Circle { center, radius }] => Ok(BoundaryPath {
                points: circle(center, *radius, max_step),
                unresolved: Vec::new(),
                warnings: Vec::new(),
            }),
            _ => Err(BoundaryError::CircleMixedWithPoints),
        };
    }
    let anchors: Vec<Coordinate> = parts
        .iter()
        .filter_map(|p| match p {
            BoundaryPart::Point(p) => Some(*p),
            _ => None,
        })
        .collect();
    let (Some(&first), Some(&last)) = (anchors.first(), anchors.last()) else {
        return Err(BoundaryError::Empty);
    };
    // 前后相邻的坐标点；越过开头或结尾时首尾相接。
    let previous = |i: usize| {
        parts[..i].iter().rev().find_map(|p| match p {
            BoundaryPart::Point(p) => Some(*p),
            _ => None,
        })
    };
    let next = |i: usize| match parts.get(i + 1) {
        Some(BoundaryPart::Point(p)) => Some(*p),
        Some(BoundaryPart::Arc { .. }) => None,
        Some(_) => parts[i + 1..].iter().find_map(|p| match p {
            BoundaryPart::Point(p) => Some(*p),
            _ => None,
        }),
        None => Some(first),
    };

    let mut points = Vec::new();
    let mut unresolved = Vec::new();
    let mut warnings = Vec::new();
    let mut leading = Vec::new();
    for (i, part) in parts.iter().enumerate() {
        match part {
            BoundaryPart::Point(p) => push_distinct(&mut points, *p),
            BoundaryPart::Unresolved(text) => {
                if points.is_empty() {
                    leading.push(text.clone());
                } else {
                    unresolved.push(UnresolvedEdge {
                        from: points.len() - 1,
                        text: text.clone(),
                    });
                }
            }
            BoundaryPart::Arc {
                center,
                radius,
                direction,
            } => {
                let start = previous(i);
                let from = start.unwrap_or(last);
                let to = next(i).ok_or(BoundaryError::ArcMissingEndpoint)?;
                for point in [from, to] {
                    let distance = get_distance(center, &point);
                    if !distance.is_finite()
                        || (distance - radius).abs() > (radius * 0.1).max(500.0)
                    {
                        warnings.push(ArcWarning {
                            point,
                            radius: *radius,
                            distance,
                        });
                    }
                }
                let vertices = arc(
                    center,
                    *radius,
                    get_azimuth(center, &from),
                    get_azimuth(center, &to),
                    *direction,
                    max_step,
                );
                if start.is_none() {
                    // 首段圆弧从最后一个点出发：先落下这个点，结尾处再回到它。
                    push_distinct(&mut points, from);
                }
                for p in &vertices[1..vertices.len() - 1] {
                    push_distinct(&mut points, *p);
                }
            }
            BoundaryPart::Circle { .. } => unreachable!(),
        }
    }
    // 以圆弧或未解析段收尾时，回到第一个点。
    let wraps = !matches!(parts.last(), Some(BoundaryPart::Point(_))) || !leading.is_empty();
    if wraps && points.last() != Some(&points[0]) {
        let from = points.len() - 1;
        points.push(points[0]);
        for text in leading {
            unresolved.push(UnresolvedEdge { from, text });
        }
    }
    Ok(BoundaryPath {
        points,
        unresolved,
        warnings,
    })
}

/// [`parse_boundary`] 加 [`boundary_path`]。
pub fn parse_boundary_path(text: &str, max_step: f64) -> Result<BoundaryPath, BoundaryError> {
    boundary_path(&parse_boundary(text)?, max_step)
}
