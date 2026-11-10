use crate::coordinate::Coordinate;

/// 获取两点之间的方位角
///
/// # 参数
///
/// - `raw` - 源坐标
/// - `target` - 目标坐标
/// - `MagVar` - 磁偏角（东正西负）
///
/// # 返回值
///
/// 方位角（度数）
pub fn get_angle(raw: &Coordinate, target: &Coordinate, mag_var: f64) -> f64 {
    const RJ: f64 = 6356752.3142;
    const RC: f64 = 6378137.0000;
    let ec = RJ + (RC - RJ) * (90.0 - raw.lat) / 90.0;
    let ed = ec * raw.lat.to_radians().cos();
    let dx = (raw.lon.to_radians() - target.lon.to_radians()) * ed;
    let dy = (target.lat.to_radians() - raw.lat.to_radians()) * ec;
    let mut angle = (dx / dy).abs().atan().to_degrees();

    let d_lo = target.lon - raw.lon;
    let d_la = target.lat - raw.lat;

    if d_lo > 0.0 && d_la <= 0.0 {
        angle = 90.0 - angle + 90.0;
    } else if d_lo <= 0.0 && d_la < 0.0 {
        angle += 180.0;
    } else if d_lo < 0.0 && d_la >= 0.0 {
        angle = 90.0 - angle + 270.0;
    }

    angle - mag_var
}

/// 获取球模型下两点之间的距离
///
/// # 参数
///
/// - `raw` - 源坐标
/// - `target` - 目标坐标
///
/// # 返回值
///
/// 距离，单位为米
pub fn get_circle_distance(raw: &Coordinate, target: &Coordinate) -> f64 {
    const EARTH_RADIUS: f64 = 6378137.0;
    let rad_lat1 = raw.lat.to_radians();
    let rad_lng1 = raw.lon.to_radians();
    let rad_lat2 = target.lat.to_radians();
    let rad_lng2 = target.lon.to_radians();
    let a = rad_lat1 - rad_lat2;
    let b = rad_lng1 - rad_lng2;
    2.0 * ((a / 2.0).sin().powi(2) + rad_lat1.cos() * rad_lat2.cos() * (b / 2.0).sin().powi(2))
        .sqrt()
        .asin()
        * EARTH_RADIUS
}

/// 获取WGS-84模型下两点之间的距离（Vincenty公式）
///
/// # 参数
///
/// - `raw` - 原坐标
/// - `target` - 目标坐标
///
/// # 返回值
///
/// 距离，单位为米
pub fn get_distance(raw: &Coordinate, target: &Coordinate) -> f64 {
    const A: f64 = 6378137.0000;
    const B: f64 = 6356752.3142;
    const F: f64 = 1.0 / 298.2572236;

    let l = (raw.lon - target.lon).to_radians();
    let u1 = ((1.0 - F) * raw.lat.to_radians().tan()).atan();
    let u2 = ((1.0 - F) * target.lat.to_radians().tan()).atan();

    let sin_u1 = u1.sin();
    let cos_u1 = u1.cos();
    let sin_u2 = u2.sin();
    let cos_u2 = u2.cos();

    let mut lambda = l;
    let mut lambda_p = std::f64::consts::PI;
    let mut cos_sq_alpha = 0.0;
    let mut sin_sigma = 0.0;
    let mut cos_2_sigma_m = 0.0;
    let mut cos_sigma = 0.0;
    let mut sigma = 0.0;

    let mut iter_count = 40;

    while (lambda - lambda_p).abs() > 1e-12 && iter_count > 0 {
        let sin_lambda = lambda.sin();
        let cos_lambda = lambda.cos();

        sin_sigma = ((cos_u2 * sin_lambda).powi(2)
            + ((cos_u1 * sin_u2 - sin_u1 * cos_u2 * cos_lambda).powi(2)))
        .sqrt();

        if sin_sigma == 0.0 {
            return 0.0; // coincident points
        }

        cos_sigma = sin_u1 * sin_u2 + cos_u1 * cos_u2 * cos_lambda;
        sigma = sin_sigma.atan2(cos_sigma);

        let alpha = (cos_u1 * cos_u2 * sin_lambda / sin_sigma).asin();
        cos_sq_alpha = alpha.cos().powi(2);

        cos_2_sigma_m = cos_sigma - 2.0 * sin_u1 * sin_u2 / cos_sq_alpha;

        let c = F / 16.0 * cos_sq_alpha * (4.0 + F * (4.0 - 3.0 * cos_sq_alpha));

        lambda_p = lambda;

        lambda = l
            + ((1.0 - c)
                * F
                * alpha.sin()
                * (sigma
                    + (c * sin_sigma
                        * (cos_2_sigma_m
                            + (c * cos_sigma * (-1.0 + 2.0 * cos_2_sigma_m * cos_2_sigma_m))))));

        iter_count -= 1;
    }

    if iter_count == 0 {
        return f64::NAN; // formula failed to converge
    }

    let u_sq = cos_sq_alpha * (A * A - B * B) / (B * B);
    let a = 1.0 + u_sq / 16384.0 * (4096.0 + u_sq * (-768.0 + u_sq * (320.0 - 175.0 * u_sq)));
    let b = u_sq / 1024.0 * (256.0 + u_sq * (-128.0 + u_sq * (74.0 - 47.0 * u_sq)));
    let delta_sigma = b
        * sin_sigma
        * (cos_2_sigma_m
            + b / 4.0
                * (cos_sigma * (-1.0 + 2.0 * cos_2_sigma_m.powi(2))
                    - b / 6.0
                        * cos_2_sigma_m
                        * (-3.0 + 4.0 * sin_sigma.powi(2))
                        * (-3.0 + 4.0 * cos_2_sigma_m.powi(2))));

    B * a * (sigma - delta_sigma)
}

/// 获取WGS-84模型下从源坐标到目标坐标的初始真方位（Vincenty反解）
///
/// 与 [`get_angle`] 的平面近似不同，这里与
/// [`get_colocated_coord`](crate::algorithm::position_transformation::get_colocated_coord)
/// 互为逆运算：按返回的方位和 [`get_distance`] 的距离正算，可以回到目标点。
///
/// # 参数
///
/// - `raw` - 源坐标
/// - `target` - 目标坐标
///
/// # 返回值
///
/// 真方位（度数，[0, 360)）。两点重合时返回 0；近对跖点不收敛时退回 [`get_angle`]。
pub fn get_azimuth(raw: &Coordinate, target: &Coordinate) -> f64 {
    const F: f64 = 1.0 / 298.2572236;

    let l = (target.lon - raw.lon).to_radians();
    let u1 = ((1.0 - F) * raw.lat.to_radians().tan()).atan();
    let u2 = ((1.0 - F) * target.lat.to_radians().tan()).atan();
    let (sin_u1, cos_u1) = u1.sin_cos();
    let (sin_u2, cos_u2) = u2.sin_cos();

    let mut lambda = l;
    for _ in 0..200 {
        let (sin_lambda, cos_lambda) = lambda.sin_cos();
        let sin_sigma = ((cos_u2 * sin_lambda).powi(2)
            + (cos_u1 * sin_u2 - sin_u1 * cos_u2 * cos_lambda).powi(2))
        .sqrt();
        if sin_sigma == 0.0 {
            return 0.0;
        }
        let cos_sigma = sin_u1 * sin_u2 + cos_u1 * cos_u2 * cos_lambda;
        let sigma = sin_sigma.atan2(cos_sigma);
        let sin_alpha = cos_u1 * cos_u2 * sin_lambda / sin_sigma;
        let cos_sq_alpha = 1.0 - sin_alpha * sin_alpha;
        let cos_2_sigma_m = if cos_sq_alpha == 0.0 {
            0.0
        } else {
            cos_sigma - 2.0 * sin_u1 * sin_u2 / cos_sq_alpha
        };
        let c = F / 16.0 * cos_sq_alpha * (4.0 + F * (4.0 - 3.0 * cos_sq_alpha));
        let previous = lambda;
        lambda = l
            + (1.0 - c)
                * F
                * sin_alpha
                * (sigma
                    + c * sin_sigma
                        * (cos_2_sigma_m + c * cos_sigma * (-1.0 + 2.0 * cos_2_sigma_m.powi(2))));
        if (lambda - previous).abs() <= 1e-12 {
            let (sin_lambda, cos_lambda) = lambda.sin_cos();
            let alpha1 = (cos_u2 * sin_lambda)
                .atan2(cos_u1 * sin_u2 - sin_u1 * cos_u2 * cos_lambda)
                .to_degrees();
            return alpha1.rem_euclid(360.0);
        }
    }
    get_angle(raw, target, 0.0).rem_euclid(360.0)
}
