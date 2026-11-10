//! 解析航图/AIP 正文中的空域边界描述。
//!
//! 例如：
//!
//! ```text
//! N373500E1125000－N373500E1130500－以 N372857E1125818 为圆心，半径 5.500km 顺时针的弧－N372300E1125000－起点
//! 以 N390520E1162740 为中心，半径 10 公里的圆
//! ```
//!
//! 边界由三类片段组成：
//!
//! - 坐标点：必须带半球标记，写法同 [`parse_flexible`]，如 `N372857E1125818`、
//!   `N37°28′57″ E112°58′18″`；
//! - 圆弧：`[以]<圆心>为圆心|为中心[，]半径<数值><单位>[的]顺时针|逆时针[方向][的]弧|弧线|圆弧`，
//!   起点和终点是它前后相邻的坐标点（最后一段圆弧的终点是边界的第一个点）；
//! - 整圆：`[以]<圆心>为圆心|为中心[，]半径<数值><单位>的圆|圆周|圆形|范围`，只能单独出现。
//!
//! 单位可写 `km`/`公里`/`千米`、`NM`/`海里`、`m`/`米`。片段之间可以用 `－ - — ~ 至 到 ， 、 ；`
//! 等分隔，`起点`、`回到起点` 之类的收尾词会被忽略。其余任何文字（如「沿国境线」）都会
//! 报错而不是被跳过，避免画出与原文不符的边界。

use crate::algorithm::measurement::{get_azimuth, get_distance};
use crate::algorithm::shape::{arc, circle, ArcDirection};
use crate::coordinate::Coordinate;
use crate::parse::{parse_flexible, AxisOrder, FlexibleParseError};
use lazy_static::lazy_static;
use regex::Regex;
use std::fmt::{Display, Formatter};

/// 边界描述中的一个片段。半径单位为米。
#[derive(Debug, Clone, Copy, PartialEq)]
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
}

/// 解析或展开边界描述时的错误。`Display` 输出中文说明。
#[derive(Debug, Clone, PartialEq)]
pub enum BoundaryError {
    /// 没有任何片段
    Empty,
    /// 出现了无法识别的文字
    UnrecognizedText(String),
    /// 坐标写法有误
    Coordinate {
        text: String,
        error: FlexibleParseError,
    },
    /// 半径不是正数
    InvalidRadius(String),
    /// 圆弧没有写顺时针还是逆时针
    ArcWithoutDirection(String),
    /// 圆弧前后缺少坐标点（例如两段圆弧相连）
    ArcMissingEndpoint,
    /// 圆弧端点到圆心的距离与半径明显不符
    ArcEndpointOffRadius { radius: f64, distance: f64 },
    /// 整圆与其他片段混在一起
    CircleMixedWithPoints,
    /// 不同的点少于 3 个，围不成区域
    TooFewPoints,
}

impl Display for BoundaryError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            BoundaryError::Empty => write!(f, "没有找到边界坐标"),
            BoundaryError::UnrecognizedText(text) => write!(f, "无法识别「{text}」"),
            BoundaryError::Coordinate { text, error } => write!(f, "坐标 {text}：{error}"),
            BoundaryError::InvalidRadius(text) => write!(f, "半径「{text}」必须大于 0"),
            BoundaryError::ArcWithoutDirection(text) => {
                write!(f, "圆弧「{text}」需注明顺时针或逆时针")
            }
            BoundaryError::ArcMissingEndpoint => write!(f, "圆弧前后都需要有坐标点"),
            BoundaryError::ArcEndpointOffRadius { radius, distance } => write!(
                f,
                "圆弧端点距圆心 {:.3} km，与半径 {:.3} km 不符",
                distance / 1000.0,
                radius / 1000.0
            ),
            BoundaryError::CircleMixedWithPoints => {
                write!(f, "整圆需单独描述，不能与其他边界点混用")
            }
            BoundaryError::TooFewPoints => write!(f, "区域至少需要 3 个不同的坐标"),
        }
    }
}

impl std::error::Error for BoundaryError {}

/// 带半球标记的坐标，纬度在前或经度在前都可以。
const COORD: &str = r#"(?:[NS]\s*\d[\d°º'′"″.:\s]*?[,，]?\s*[EW]\s*\d[\d°º'′"″.:]*(?:\s+\d[\d°º'′"″.:]*){0,2}|[EW]\s*\d[\d°º'′"″.:\s]*?[,，]?\s*[NS]\s*\d[\d°º'′"″.:]*(?:\s+\d[\d°º'′"″.:]*){0,2})"#;

lazy_static! {
    static ref RE_COORD: Regex = Regex::new(COORD).unwrap();
    static ref RE_CLAUSE: Regex = Regex::new(&format!(
        r"(?:以\s*)?(?P<center>{COORD})\s*为\s*(?:圆心|中心)\s*[，,、]?\s*(?:的)?\s*半径\s*(?:为|是)?\s*(?P<radius>\d+(?:\.\d+)?)\s*(?P<unit>KM|公里|千米|NM|海里|M|米)\s*(?:的)?\s*[，,、]?\s*(?P<direction>顺时针|逆时针)?\s*(?:方向)?\s*(?:的)?\s*(?P<shape>圆弧|弧线|弧|圆周|圆形|圆|范围)"
    ))
    .unwrap();
    static ref RE_FILLER: Regex =
        Regex::new(r"^(?:[\s\-－—–~～至到，,、；;。:：()（）]|回到|返回|起点|终点|闭合|止)*$").unwrap();
}

fn check_gap(gap: &str) -> Result<(), BoundaryError> {
    if RE_FILLER.is_match(gap) {
        Ok(())
    } else {
        Err(BoundaryError::UnrecognizedText(
            gap.trim().chars().take(24).collect(),
        ))
    }
}

fn coordinate(text: &str) -> Result<Coordinate, BoundaryError> {
    parse_flexible(text, AxisOrder::LatLon).map_err(|error| BoundaryError::Coordinate {
        text: text.trim().to_string(),
        error,
    })
}

/// 把边界描述拆成片段，不做几何展开。
pub fn parse_boundary(text: &str) -> Result<Vec<BoundaryPart>, BoundaryError> {
    let text = text.to_uppercase();
    let mut parts = Vec::new();
    let mut position = 0;
    loop {
        let clause = RE_CLAUSE.captures_at(&text, position);
        let point = RE_COORD.find_at(&text, position);
        let clause_start = clause.as_ref().map(|c| c.get(0).unwrap().start());
        let take_clause = match (clause_start, point.map(|p| p.start())) {
            (None, None) => break,
            (Some(_), None) => true,
            (None, Some(_)) => false,
            (Some(c), Some(p)) => c <= p,
        };
        if take_clause {
            let captures = clause.unwrap();
            let whole = captures.get(0).unwrap();
            check_gap(&text[position..whole.start()])?;
            let center = coordinate(&captures["center"])?;
            let value: f64 = captures["radius"].parse().unwrap_or(0.0);
            let radius = value
                * match &captures["unit"] {
                    "KM" | "公里" | "千米" => 1000.0,
                    "NM" | "海里" => 1852.0,
                    _ => 1.0,
                };
            if !(radius.is_finite() && radius > 0.0) {
                return Err(BoundaryError::InvalidRadius(format!(
                    "{}{}",
                    &captures["radius"], &captures["unit"]
                )));
            }
            let is_arc = captures["shape"].contains('弧');
            let direction = captures.name("direction").map(|d| match d.as_str() {
                "顺时针" => ArcDirection::Clockwise,
                _ => ArcDirection::CounterClockwise,
            });
            parts.push(match (is_arc, direction) {
                (true, Some(direction)) => BoundaryPart::Arc {
                    center,
                    radius,
                    direction,
                },
                (true, None) => {
                    return Err(BoundaryError::ArcWithoutDirection(
                        whole.as_str().to_string(),
                    ))
                }
                (false, _) => BoundaryPart::Circle { center, radius },
            });
            position = whole.end();
        } else {
            let point = point.unwrap();
            check_gap(&text[position..point.start()])?;
            parts.push(BoundaryPart::Point(coordinate(point.as_str())?));
            position = point.end();
        }
    }
    check_gap(&text[position..])?;
    if parts.is_empty() {
        return Err(BoundaryError::Empty);
    }
    Ok(parts)
}

fn push_distinct(ring: &mut Vec<Coordinate>, point: Coordinate) {
    if ring.last() != Some(&point) {
        ring.push(point);
    }
}

fn check_on_arc(center: &Coordinate, radius: f64, point: &Coordinate) -> Result<(), BoundaryError> {
    let distance = get_distance(center, point);
    // 航图坐标通常只精确到秒（约 30 m），半径也常取整，所以只拦明显的错配。
    if !distance.is_finite() || (distance - radius).abs() > (radius * 0.1).max(500.0) {
        return Err(BoundaryError::ArcEndpointOffRadius { radius, distance });
    }
    Ok(())
}

/// 把片段展开成闭合的多边形顶点（首尾相同）。
///
/// 圆弧按 WGS-84 正解生成，相邻顶点的圆心角不超过 `max_step` 度；圆弧两端直接使用
/// 原文给出的坐标点。
pub fn boundary_ring(
    parts: &[BoundaryPart],
    max_step: f64,
) -> Result<Vec<Coordinate>, BoundaryError> {
    if parts.is_empty() {
        return Err(BoundaryError::Empty);
    }
    if parts
        .iter()
        .any(|p| matches!(p, BoundaryPart::Circle { .. }))
    {
        return match parts {
            [BoundaryPart::Circle { center, radius }] => Ok(circle(center, *radius, max_step)),
            _ => Err(BoundaryError::CircleMixedWithPoints),
        };
    }
    let point_at = |i: usize| match parts.get(i) {
        Some(BoundaryPart::Point(p)) => Some(*p),
        _ => None,
    };
    let mut ring = Vec::new();
    for (i, part) in parts.iter().enumerate() {
        match part {
            BoundaryPart::Point(p) => push_distinct(&mut ring, *p),
            BoundaryPart::Arc {
                center,
                radius,
                direction,
            } => {
                // 首段圆弧从最后一个点出发，末段圆弧回到第一个点。
                let from = if i == 0 {
                    point_at(parts.len() - 1)
                } else {
                    point_at(i - 1)
                }
                .ok_or(BoundaryError::ArcMissingEndpoint)?;
                let to = if i + 1 == parts.len() {
                    point_at(0)
                } else {
                    point_at(i + 1)
                }
                .ok_or(BoundaryError::ArcMissingEndpoint)?;
                check_on_arc(center, *radius, &from)?;
                check_on_arc(center, *radius, &to)?;
                let points = arc(
                    center,
                    *radius,
                    get_azimuth(center, &from),
                    get_azimuth(center, &to),
                    *direction,
                    max_step,
                );
                for p in &points[1..points.len() - 1] {
                    push_distinct(&mut ring, *p);
                }
            }
            BoundaryPart::Circle { .. } => unreachable!(),
        }
    }
    let mut distinct: Vec<Coordinate> = Vec::new();
    for p in &ring {
        if !distinct.contains(p) {
            distinct.push(*p);
        }
    }
    if distinct.len() < 3 {
        return Err(BoundaryError::TooFewPoints);
    }
    if ring.first() != ring.last() {
        ring.push(ring[0]);
    }
    Ok(ring)
}

/// [`parse_boundary`] 加 [`boundary_ring`]。
pub fn parse_boundary_ring(text: &str, max_step: f64) -> Result<Vec<Coordinate>, BoundaryError> {
    boundary_ring(&parse_boundary(text)?, max_step)
}
