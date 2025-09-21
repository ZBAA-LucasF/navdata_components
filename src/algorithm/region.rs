use crate::Coordinate;

/// 射线法判断一个坐标点是否在区域内
///
/// # 参数
///
///  - `point` - 需要判断的坐标点
///  - `area` - 指定的区域，由坐标组成
///
/// # 返回值
///
/// 目标点是否在指定区域内
pub fn is_point_in_area(point: &Coordinate, area: &[Coordinate]) -> bool {
    let a_lat = point.lat;
    let a_lon = point.lon;
    let mut i_sum = 0;

    let i_count = area.len();
    if i_count < 3 {
        return false;
    }

    for i in 0..i_count {
        let (d_lat1, d_lon1, d_lat2, d_lon2) = if i == i_count - 1 {
            (area[i].lat, area[i].lon, area[0].lat, area[0].lon)
        } else {
            (area[i].lat, area[i].lon, area[i + 1].lat, area[i + 1].lon)
        };

        if ((a_lat >= d_lat1) && (a_lat < d_lat2)) || ((a_lat >= d_lat2) && (a_lat < d_lat1)) {
            if (d_lat1 - d_lat2).abs() > 0.0 {
                let d_lon = d_lon1 - ((d_lon1 - d_lon2) * (d_lat1 - a_lat)) / (d_lat1 - d_lat2);
                if d_lon < a_lon {
                    i_sum += 1;
                }
            }
        }
    }

    i_sum % 2 != 0
}
