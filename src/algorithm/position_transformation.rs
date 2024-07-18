use crate::coordinate::Coordinate;
use std::f64::consts::PI;

/// 生成当前坐标按给定的方向和距离移动后的坐标
///
/// # 参数
///
///  - `bearing` - 方向，使用磁方向时输入磁偏角
///  - `distance` - 距离，单位为米。
///  - `MagVar` - 磁偏角（东正西负）。使用真方向时为0。
///
/// # 返回值
///
/// 经过移动后的坐标
pub fn get_colocated_coord(
    raw: &Coordinate,
    bearing: f64,
    distance: f64,
    mag_var: f64,
) -> Coordinate {
    let mut temp = Coordinate { lat: 0.0, lon: 0.0 };

    const A: f64 = 6378137.0000;
    const B: f64 = 6356752.3142;
    const F: f64 = 1.0 / 298.2572236;
    let alpha1 = (bearing + mag_var).to_radians();
    let sin_alpha1 = alpha1.sin();
    let cos_alpha1 = alpha1.cos();
    let tan_u1 = (1.0 - F) * (raw.lat.to_radians()).tan();
    let cos_u1 = 1.0 / (1.0 + tan_u1 * tan_u1).sqrt();
    let sin_u1 = tan_u1 * cos_u1;
    let theta1 = (tan_u1 / cos_alpha1).atan();
    let sin_alpha = cos_u1 * sin_alpha1;
    let cos_2_alpha = 1.0 - sin_alpha * sin_alpha;
    let u2 = cos_2_alpha * (A * A - B * B) / (B * B);
    let aa = 1.0 + (u2 / 16384.0 * (4096.0 + u2 * (-768.0 + u2 * (320.0 - 175.0 * u2))));
    let bb = u2 / 1024.0 * (256.0 + u2 * (-128.0 + u2 * (74.0 - 47.0 * u2)));
    let mut cos_2_theta_m = 0.0;
    let mut sin_theta = 0.0;
    let mut cos_theta = 0.0;
    let mut theta = distance / (B * aa);
    let mut tau = 2.0 * PI;
    while (theta - tau).abs() > 1E-12 {
        cos_2_theta_m = ((2.0 * theta1) + theta).cos();
        sin_theta = theta.sin();
        cos_theta = theta.cos();
        let delta_theta = bb
            * sin_theta
            * (cos_2_theta_m
                + bb / 4.0
                    * (cos_theta * (-1.0 + 2.0 * cos_2_theta_m * cos_2_theta_m)
                        - bb / 6.0
                            * cos_2_theta_m
                            * (-3.0 + 4.0 * sin_theta * sin_theta)
                            * (-3.0 + 4.0 * cos_2_theta_m * cos_2_theta_m)));
        tau = theta;
        theta = distance / (B * aa) + delta_theta;
    }
    let t = sin_u1 * sin_theta - cos_u1 * cos_theta * cos_alpha1;
    let lat2 = (sin_u1 * cos_theta + cos_u1 * sin_theta * cos_alpha1)
        .atan2((1.0 - F) * ((sin_alpha * sin_alpha + t * t).sqrt()));
    let lambda =
        (sin_theta * sin_alpha1).atan2(cos_u1 * cos_theta - sin_u1 * sin_theta * cos_alpha1);
    let c = F / 16.0 * cos_2_alpha * (4.0 + F * (4.0 - 3.0 * cos_2_alpha));
    let l = lambda
        - (1.0 - c)
            * F
            * sin_alpha
            * (theta
                + c * sin_theta
                    * (cos_2_theta_m
                        + c * cos_theta * (-1.0 + 2.0 * cos_2_theta_m * cos_2_theta_m)));
    temp.lat = lat2.to_degrees();
    temp.lon = raw.lon + l.to_degrees();
    temp
}
