use lazy_static::lazy_static;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, str::FromStr};

#[derive(Debug, PartialEq, Serialize, Deserialize, Copy, Clone)]
pub struct Coordinate {
    /// 纬度
    pub lat: f64,
    /// 经度
    pub lon: f64,
}

lazy_static! {
    ///  # 表示坐标格式的正则表达式（单个数值）
    ///  - 第一组是正负
    ///  - 第二组是度
    ///  - 第三组是分
    ///  - 第四组是秒
    static ref RE_SINGLE: Vec<Regex> = vec![
        Regex::new(r"^([NSEW])([01]?\d{2})(\d{2}\.\d*)$").unwrap(),
        Regex::new(r"^([NSEW])([01]?\d{2})(\d{2})(\d{2}\.\d*)$").unwrap(),
        Regex::new(r"^([NSEW])([01]?\d{2})(\d{2})(\d{2})$").unwrap(),
        Regex::new(r"^([NSEW])(\d{3})\.(\d{2})\.(\d{2}\.\d{3})$").unwrap(),
        Regex::new(r"^([NSEW]) ([01]?\d{2}) (\d{2}\.\d*)$").unwrap(),
        Regex::new(r"^([+-]?)(\d{2,3}\.\d*)$").unwrap(),
    ];

    /// 表示单位换算时的比例
    static ref RATIO: HashMap<usize, f64> = vec![(2, 1.0), (3, 1.0 / 60.0), (4, 1.0 / 3600.0)]
        .into_iter()
        .collect();

    /// 表示坐标格式的正则表达式（经纬度都包含）
    static ref RE_COMBINE: Vec<Regex> = vec![
        Regex::new(r"^([NS].*?) ?([EW].*?)$").unwrap(),  // Nxxxx.xExxxxx.x、Nxxxxxx.xExxxxxxx.x、NxxxxxxExxxxxxx、Nxxx.xx.xx.xxx Exxx.xx.xx.xxx、N xx xx.xxxxxx E xxx xx.xxxxxx
        Regex::new(r"^(.*?),(.*?)$").unwrap(),         // xx.xxxxxxxxxxxxxx,xxx.xxxxxxxxxxxxxxx
        Regex::new(r"^([^ ]*?) ([^ ]*?)$").unwrap()    // +xx.xxxxxxxx +xxx.xxxxxx
    ];
}

/// 处理坐标时可能出现的错误类型
#[derive(Debug, PartialEq)]
pub enum CoordParseError {
    /// 纬度处理错误
    LatParseError,
    /// 经度处理错误
    LonParseError,
    /// 没有匹配的格式
    NoMatchingFormat,
}

/// 将输入的字符串变成f64类型的数值
fn parse(s: &str) -> Option<f64> {
    let mut result: f64 = 0.0;

    for r in RE_SINGLE.iter() {
        if r.is_match(s) {
            let group = r.captures(s).unwrap();
            for i in 2..group.len() {
                result +=
                    group.get(i).unwrap().as_str().parse::<f64>().unwrap() * RATIO.get(&i).unwrap();
            }

            result *= match group.get(1).unwrap().as_str() {
                "S" | "W" | "-" => -1.0,
                _ => 1.0,
            };

            return Some(result);
        }
    }
    None
}

impl FromStr for Coordinate {
    type Err = CoordParseError;

    /// 通过字符串创建Coordinate
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        for r in RE_COMBINE.iter() {
            if r.is_match(s) {
                let group = r.captures(s).unwrap();
                return Ok(Coordinate {
                    lat: match parse(group.get(1).unwrap().as_str()) {
                        Some(x) => x,
                        None => return Err(Self::Err::LatParseError),
                    },
                    lon: match parse(group.get(2).unwrap().as_str()) {
                        Some(x) => x,
                        None => return Err(Self::Err::LonParseError),
                    },
                });
            }
        }
        Err(Self::Err::NoMatchingFormat)
    }
}

impl Eq for Coordinate {}

impl Coordinate {
    /// 纬度值的度部分
    pub fn lat_d(&self) -> i32 {
        self.lat.trunc() as i32
    }
    /// 经度值的度部分
    pub fn lon_d(&self) -> i32 {
        self.lon.trunc() as i32
    }

    /// 纬度值的分部分
    pub fn lat_m(&self) -> i32 {
        (60.0 * (self.lat.abs() - self.lat.abs().trunc())).trunc() as i32
    }

    /// 经度值的分部分
    pub fn lon_m(&self) -> i32 {
        (60.0 * (self.lon.abs() - self.lon.abs().trunc())).trunc() as i32
    }

    /// 纬度值的秒部分（取整值）
    pub fn lat_s(&self) -> f64 {
        (60.0
            * ((60.0 * self.lat.abs() - self.lat.abs().trunc())
                - (60.0 * self.lat.abs() - self.lat.abs().trunc()).trunc()))
        .round()
    }
    /// 纬度值的秒部分（取整值）
    pub fn lon_s(&self) -> f64 {
        (60.0
            * ((60.0 * self.lon.abs() - self.lon.abs().trunc())
                - (60.0 * self.lon.abs() - self.lon.abs().trunc()).trunc()))
        .round()
    }

    /// 纬度值的秒部分（完全值）
    pub fn lat_sp(&self) -> f64 {
        60.0 * ((60.0 * self.lat.abs() - self.lat.abs().trunc())
            - (60.0 * self.lat.abs() - self.lat.abs().trunc()).trunc())
    }
    /// 经度值的秒部分（完全值）
    pub fn lon_sp(&self) -> f64 {
        60.0 * ((60.0 * self.lon.abs() - self.lon.abs().trunc())
            - (60.0 * self.lon.abs() - self.lon.abs().trunc()).trunc())
    }

    /// 纬度值的分部分（包含秒，小数形式）
    pub fn lat_md(&self) -> f64 {
        self.lat.fract() * 60.0
    }

    /// 经度值的分部分（包含秒，小数形式）
    pub fn lon_md(&self) -> f64 {
        self.lon.fract() * 60.0
    }
}
