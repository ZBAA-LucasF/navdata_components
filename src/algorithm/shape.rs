//! 以 WGS-84 正解生成圆、圆弧和扇形的顶点。
//!
//! 方位一律是真方位（度，正北为 0，顺时针增大），半径单位为米。输出的经度规整
//! 到 [-180, 180)，靠近日界线的图形不会因为 181° 之类的值被下游拒绝。

use crate::algorithm::position_transformation::get_colocated_coord;
use crate::coordinate::Coordinate;

/// 圆弧的走向。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArcDirection {
    /// 顺时针（方位递增，航图中的「右转」）
    Clockwise,
    /// 逆时针（方位递减）
    CounterClockwise,
}

impl ArcDirection {
    /// 从 `start` 到 `end` 沿此方向扫过的角度，顺时针为正、逆时针为负。
    ///
    /// 起止方位相同时视为整圈（±360°）。
    pub fn sweep(self, start: f64, end: f64) -> f64 {
        let sweep = match self {
            ArcDirection::Clockwise => (end - start).rem_euclid(360.0),
            ArcDirection::CounterClockwise => -(start - end).rem_euclid(360.0),
        };
        if sweep == 0.0 {
            match self {
                ArcDirection::Clockwise => 360.0,
                ArcDirection::CounterClockwise => -360.0,
            }
        } else {
            sweep
        }
    }
}

fn normalized(p: Coordinate) -> Coordinate {
    Coordinate {
        lat: p.lat,
        lon: (p.lon + 180.0).rem_euclid(360.0) - 180.0,
    }
}

/// 沿圆弧生成顶点，包含起点和终点。
///
/// # 参数
///
/// - `center` - 圆心
/// - `radius` - 半径，单位为米
/// - `start_bearing` / `end_bearing` - 圆心到起点、终点的真方位
/// - `direction` - 走向；起止方位相同时生成整圈
/// - `max_step` - 相邻顶点间的最大圆心角（度），不大于 0 时按 1°
///
/// 顶点数为 `ceil(|扫过角度| / max_step) + 1`，且至少 3 个。
pub fn arc(
    center: &Coordinate,
    radius: f64,
    start_bearing: f64,
    end_bearing: f64,
    direction: ArcDirection,
    max_step: f64,
) -> Vec<Coordinate> {
    let max_step = if max_step > 0.0 { max_step } else { 1.0 };
    let sweep = direction.sweep(start_bearing, end_bearing);
    let steps = ((sweep.abs() / max_step).ceil() as usize).max(2);
    (0..=steps)
        .map(|i| {
            normalized(get_colocated_coord(
                center,
                start_bearing + sweep * i as f64 / steps as f64,
                radius,
                0.0,
            ))
        })
        .collect()
}

/// 生成闭合的圆（首尾是同一个点）。
pub fn circle(center: &Coordinate, radius: f64, max_step: f64) -> Vec<Coordinate> {
    let mut points = arc(center, radius, 0.0, 0.0, ArcDirection::Clockwise, max_step);
    let last = points.len() - 1;
    points[last] = points[0];
    points
}

/// 生成闭合的扇形：圆弧，再连回圆心，最后回到圆弧起点。
pub fn sector(
    center: &Coordinate,
    radius: f64,
    start_bearing: f64,
    end_bearing: f64,
    direction: ArcDirection,
    max_step: f64,
) -> Vec<Coordinate> {
    let mut points = arc(
        center,
        radius,
        start_bearing,
        end_bearing,
        direction,
        max_step,
    );
    points.push(normalized(*center));
    points.push(points[0]);
    points
}
