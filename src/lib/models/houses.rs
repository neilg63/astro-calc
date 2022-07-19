use serde::{Serialize, Deserialize};
use libswe_sys::swerust::{handler_swe14::*};
use super::{geo_pos::*};
use super::super::core::{calc_altitude};

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub struct AscMc {
  ascendant: f64,
  mc: f64,
  armc: f64,
  vertex: f64,
  equasc: f64,		// "equatorial ascendant" *
  coasc1: f64,		// "co-ascendant" (W. Koch) *
  coasc2: f64,		// "co-ascendant" (M. Munkasey) *
  polasc: f64,
  #[serde(rename="mcAlt",skip_serializing_if = "Option::is_none")]
  mc_alt: Option<f64>
}

impl AscMc {
  pub fn new(points: [f64; 10], mc_alt: Option<f64>) -> AscMc {
      AscMc {
        ascendant: points[0],
        mc: points[1],
        armc: points[2],
        vertex: points[3],
        equasc: points[4],
        coasc1: points[5],
        coasc2: points[6],
        polasc: points[7],
        mc_alt
      }
  }

  pub fn add_mc_alt(mut self, mc_alt: f64) {
    self.mc_alt = Some(mc_alt);
  }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct HouseData {
  pub jd: f64,
  pub lat: f64,
  pub lng: f64,
  pub system: char,
  pub houses: Vec<f64>,
  pub points: AscMc
}

impl HouseData {

  pub fn new(jd: f64, lat: f64, lng: f64, system: char, calc_mc_alt: bool) -> HouseData {
    let hd = houses(jd, lat, lng, system);
    let houses: Vec<f64> = match system {
      'G' => hd.cusps[1..37].to_vec(),
      _ => hd.cusps[1..13].to_vec(),
    };
    let mc_alt = if calc_mc_alt && hd.ascmc.len() > 0 { Some(calc_altitude(jd, false, lat, lng, hd.ascmc[1], 0f64)) } else { None };
      HouseData {
        jd: jd,
        lng: lng,
        lat: lat,
        system: system,
        houses,
        points: AscMc::new(hd.ascmc, mc_alt)
    }
  }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct HouseSet {
  pub system: char,
  pub houses: Vec<f64>
}

impl HouseSet {
  pub fn new(system: char, houses: Vec<f64>) -> HouseSet { 
    HouseSet{system, houses}
  }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct HouseSetData {
  pub points: AscMc,
  pub sets: Vec<HouseSet>,
}

impl HouseSetData {
  pub fn new(points: AscMc, sets: Vec<HouseSet>) -> HouseSetData { 
    HouseSetData{ points, sets }
  }
}

pub fn get_ascendant(jd: f64, lat: f64, lng: f64) -> f64 {
  let hd = houses(jd, lat, lng, 'W');
  hd.ascmc[0]
}

pub fn calc_ascendant(jd: f64, geo: GeoPos) -> f64 {
  get_ascendant(jd, geo.lat, geo.lng)
}

pub fn get_house_data(jd: f64, lat: f64, lng: f64, system: char, calc_mc_alt: bool) -> HouseData {
  HouseData::new(jd, lat, lng, system, calc_mc_alt)
}

pub fn houses_system_chars() -> Vec<char> {
  vec!['W','E','O','P','K','B','C','M','R','T','A','X','G','H']
}

pub fn match_house_systems_chars(ref_str: String) -> Vec<char> {
  let ref_chars: Vec<char> = ref_str.split(",").filter(|s| s.len() > 0).map(|c| c.to_uppercase().chars().nth(0).unwrap()).collect();
  let all_chars = houses_system_chars();
  ref_chars.iter().filter(|c| all_chars.contains(c)).map(|c| *c).collect::<Vec<char>>()
}

pub fn get_house_systems(jd: f64, geo: GeoPos, keys: Vec<char>) -> HouseSetData {
  let house_systems:Vec<char> = houses_system_chars();
  let match_all = keys.len() == 1 && keys[0] == 'a';
  let match_whole_only = keys.len() == 1 && keys[0] == 'W' || keys.len() < 1;
  let matched_keys = if match_whole_only { vec!['W'] } else { keys };
  let mut points: AscMc = AscMc::new([0f64; 10], None);
  let mut points_matched = false;
  let mut sets: Vec<HouseSet> = Vec::new();
  for key in house_systems {
    let hd = get_house_data(jd, geo.lat, geo.lng, key, !points_matched);
    if match_all || matched_keys.contains(&key) {
      if !points_matched {
        points = hd.points;
        points_matched = true;
      }
      
      sets.push(HouseSet::new(key, hd.houses))
    }
  }
  HouseSetData::new(points, sets)
}

pub fn get_all_house_systems(jd: f64, geo: GeoPos) -> HouseSetData {
  get_house_systems(jd, geo, vec!['a'])
}
