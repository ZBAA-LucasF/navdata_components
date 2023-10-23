use lazy_static::lazy_static;
use regex::Regex;
use std::{collections::HashMap, str::FromStr};

#[derive(Debug, PartialEq)]
pub struct Coordinate {
    pub lat: f64,
    pub lon: f64,
}

/// 将输入的字符串变成f64类型的数值
fn parse(s: &str) -> Option<f64> {
    lazy_static! {
        static ref RE: Vec<Regex> = vec![
            Regex::new(r"^([NSEW])([01]?\d{2})(\d{2}\.\d*)$").unwrap(),
            Regex::new(r"^([NSEW])([01]?\d{2})(\d{2})(\d{2}\.\d*)$").unwrap(),
            Regex::new(r"^([NSEW])([01]?\d{2})(\d{2})(\d{2})$").unwrap(),
            Regex::new(r"^([NSEW])(\d{3})\.(\d{2})\.(\d{2}\.\d{3})$").unwrap(),
            Regex::new(r"^([NSEW]) ([01]?\d{2}) (\d{2}\.\d*)$").unwrap(),
            Regex::new(r"^([+-]?)(\d{2,3}\.\d*)$").unwrap(),
        ];
        static ref RATIO: HashMap<usize, f64> = vec![(2, 1.0), (3, 1.0 / 60.0), (4, 1.0 / 3600.0)]
            .into_iter()
            .collect();
    }

    let mut result: f64 = 0.0;

    for r in RE.iter() {
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
    type Err = &'static str;

    /// 通过字符串创建Coordinate
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        lazy_static! {
            static ref RE: Vec<Regex> = vec![
            Regex::new(r"^([NS].*?) ?([EW].*?)$").unwrap(),  // Nxxxx.xExxxxx.x、Nxxxxxx.xExxxxxxx.x、NxxxxxxExxxxxxx、Nxxx.xx.xx.xxx Exxx.xx.xx.xxx、N xx xx.xxxxxx E xxx xx.xxxxxx
            Regex::new(r"^(.*?),(.*?)$").unwrap(),         // xx.xxxxxxxxxxxxxx,xxx.xxxxxxxxxxxxxxx
            Regex::new(r"^([^ ]*?) ([^ ]*?)$").unwrap()    // +xx.xxxxxxxx +xxx.xxxxxx
            ];
        }

        for r in RE.iter() {
            if r.is_match(s) {
                let group = r.captures(s).unwrap();
                return Ok(Coordinate {
                    lat: match parse(group.get(1).unwrap().as_str()) {
                        Some(x) => x,
                        None => return Err("lat parse error"),
                    },
                    lon: match parse(group.get(2).unwrap().as_str()) {
                        Some(x) => x,
                        None => return Err("lon parse error"),
                    },
                });
            }
        }
        Err("Failed")
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

