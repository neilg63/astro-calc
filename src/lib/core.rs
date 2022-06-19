use math::round::{floor};
use libswe_sys::sweconst::{Bodies, OptionalFlag};
use libswe_sys::swerust::{handler_swe03::*};
use super::settings::{ayanamshas::*};
use super::traits::*;
use super::models::{graha_pos::*, geo_pos::*, general::*, houses::{calc_ascendant}};
use super::super::extensions::swe::{azalt, set_topo, set_sid_mode, get_ayanamsha};
use std::collections::{HashMap};

pub fn calc_body_jd(jd: f64, sample_key: &str, sidereal: bool, topo: bool) -> GrahaPos {
  let combo: i32;
  let speed_flag = OptionalFlag::Speed as i32;
  if topo {
    let topo_flag = OptionalFlag::TopocentricPosition as i32;
    if sidereal {
      combo = speed_flag | OptionalFlag::SideralPosition as i32 | topo_flag;
    } else {
      combo = speed_flag | topo_flag;
    }
  } else {
    if sidereal {
      combo = speed_flag | OptionalFlag::SideralPosition as i32;
    } else {
      combo = speed_flag;
    }
  }
  let result = calc_ut(jd, Bodies::from_key(sample_key), combo);
  GrahaPos::new(sample_key, result.longitude, result.latitude, result.speed_longitude, result.speed_latitude)
}

/**
 * Only implement tropical variants for equatorial positions
 * Ayanamsha value may be subtracted if required
 */
pub fn calc_body_eq_jd(jd: f64, key: &str, topo: bool) -> GrahaPos {
  let combo: i32;
  //let eq_flag = OptionalFlag::SEFLG_EQUATORIAL;
  let eq_flag = OptionalFlag::EquatorialPosition as i32;
  let speed_flag = OptionalFlag::Speed as i32;
  if topo {
    combo = speed_flag | OptionalFlag::TopocentricPosition as i32 | eq_flag;
  } else {
    combo = speed_flag | eq_flag;
  }
  let result = calc_ut(jd, Bodies::from_key(key), combo);
  GrahaPos::new_eq(key, result.longitude, result.latitude, result.speed_longitude, result.speed_latitude)
}

pub fn calc_body_dual_jd(jd: f64, key: &str, topo: bool) -> GrahaPos {
  let combo: i32;
  //let eq_flag = OptionalFlag::SEFLG_EQUATORIAL;
  let eq_flag = OptionalFlag::EquatorialPosition as i32;
  let speed_flag = OptionalFlag::Speed as i32;
  if topo {
    combo = speed_flag | OptionalFlag::TopocentricPosition as i32 | eq_flag;
  } else {
    combo = speed_flag | eq_flag;
  }
  let combo_geo = if topo { speed_flag | OptionalFlag::TopocentricPosition as i32 } else { speed_flag };
  let result = calc_ut(jd, Bodies::from_key(key), combo);
  let result_geo = calc_ut(jd, Bodies::from_key(key), combo_geo);
  GrahaPos::new_both(key, result_geo.longitude, result_geo.latitude, result.longitude, result.latitude, result.speed_longitude, result.speed_latitude)
}

pub fn calc_body_dual_jd_geo(jd: f64, key: &str) -> GrahaPos {
  calc_body_dual_jd(jd, key, false)
}

pub fn calc_body_dual_jd_topo(jd: f64, key: &str, geo: GeoPos) -> GrahaPos {
  set_topo(geo.lat, geo.lng, geo.alt);
  calc_body_dual_jd(jd, key, true)
}

pub fn calc_body_eq_jd_topo(jd: f64, key: &str, geo: GeoPos) -> GrahaPos {
  set_topo(geo.lat, geo.lng, geo.alt);
  calc_body_eq_jd(jd, key, true)
}

/*
 Get tropical geocentric coordinates
*/
pub fn calc_body_jd_geo(jd: f64, key: &str) -> GrahaPos {
  calc_body_jd(jd, key, false, false)
}

/*
 Get set of tropical geocentric coordinates for one celestial body
*/
pub fn calc_body_positions_jd_geo(jd_start: f64, key: &str, days: i32, num_per_day: f64) -> Vec<GrahaPosItem> {
  let mut items: Vec<GrahaPosItem> = Vec::new();
  let max_f64 = floor(days as f64 * num_per_day, 0);
  let max = max_f64 as i32;
  let increment = 1f64 / num_per_day;
  for i in 0..max {
    let curr_jd = jd_start + (i as f64 * increment);
    let graha_pos = calc_body_jd_geo(curr_jd, key);
    items.push(GrahaPosItem::new(curr_jd, graha_pos));
  }
  items
}

/*
 Get set of tropical geocentric coordinates for groups of celestial bodies
*/
pub fn calc_bodies_positions_jd_geo(jd_start: f64, keys: Vec<&str>, days: i32, num_per_day: f64) -> Vec<GrahaPosSet> {
  let mut items: Vec<GrahaPosSet> = Vec::new();
  let max_f64 = floor(days as f64 * num_per_day, 0);
  let max = max_f64 as i32;
  let increment = 1f64 / num_per_day;
  for i in 0..max {
    let curr_jd = jd_start + (i as f64 * increment);
    let mut bodies: Vec<GrahaPos> = Vec::new();
    for key in &keys {
      let graha_pos = calc_body_jd_geo(curr_jd, key);
      bodies.push(graha_pos);
    }
    items.push(GrahaPosSet::new(curr_jd, bodies));
  }
  items
}

/*
 Get sidereal geocentric coordinates with an ayanamsha key
*/
pub fn calc_body_jd_geo_sidereal(jd: f64, key: &str, aya_key: &str) -> GrahaPos {
  set_sid_mode(Ayanamsha::from_key(aya_key).as_i32());
  calc_body_jd(jd, key, true, false)
}

/*
 Get tropical topocentric coordinates with geo-coordinates
*/
pub fn calc_body_jd_topo(jd: f64, key: &str, geo: GeoPos) -> GrahaPos {
  set_topo(geo.lat, geo.lng, geo.alt);
  calc_body_jd(jd, key, false, true)
}

/*
 Get sidereal topocentric coordinates with geo-coordinates and an ayanamsha key
*/
pub fn calc_body_jd_topo_sidereal(jd: f64, key: &str, geo: GeoPos, aya_key: &str) -> GrahaPos {
  set_topo(geo.lat, geo.lng, geo.alt);
  set_sid_mode(Ayanamsha::from_key(aya_key).as_i32());
  calc_body_jd(jd, key, false, true)
}

/*
  Fetch a set of
*/
pub fn get_bodies_dual_geo(jd: f64, keys: Vec<&str>) -> Vec<GrahaPos> {
  let mut bodies: Vec<GrahaPos> = Vec::new();
  for key in keys {
    let result = calc_body_dual_jd_geo(jd, key);
    bodies.push(result);
  }
  bodies
}

pub fn get_bodies_ecl_geo(jd: f64, keys: Vec<&str>) -> Vec<GrahaPos> {
  let mut bodies: Vec<GrahaPos> = Vec::new();
  for key in keys {
    let result = calc_body_jd_geo(jd, key);
    bodies.push(result);
  }
  bodies
}

pub fn get_bodies_ecl_topo(jd: f64, keys: Vec<&str>, geo: GeoPos) -> Vec<GrahaPos> {
  let mut bodies: Vec<GrahaPos> = Vec::new();
  for key in keys {
    let result = calc_body_jd_topo(jd, key, geo);
    bodies.push(result);
  }
  bodies
}

pub fn get_body_longitudes(jd: f64, geo: GeoPos, mode: &str) -> HashMap<String, f64> {
  let mut items: HashMap<String, f64> = HashMap::new();
  let keys = vec!["su", "mo", "ma", "me", "ju", "ve", "sa", "ur", "ne", "pl", "ke"];
  let bodies = match mode {
    "topo" => get_bodies_ecl_topo(jd, keys, geo),
    _ => get_bodies_ecl_geo(jd, keys),
  };
  items.insert("as".to_string(), calc_ascendant(jd, geo));
  for body in bodies {
    items.insert(body.key, body.lng);
  }
  items
}


pub fn get_body_longitudes_geo(jd: f64, geo: GeoPos) -> HashMap<String, f64> {
  get_body_longitudes(jd, geo, "geo")
}

pub fn get_body_longitudes_topo(jd: f64, geo: GeoPos) -> HashMap<String, f64> {
  get_body_longitudes(jd, geo, "topo")
}

pub fn get_bodies_dual_topo(jd: f64, keys: Vec<&str>, geo: GeoPos) -> Vec<GrahaPos> {
  let mut bodies: Vec<GrahaPos> = Vec::new();
  for key in keys {
    let result = calc_body_dual_jd_topo(jd, key, geo);
    bodies.push(result);
  }
  bodies
}

/*
* Match the projected altitude of any celestial object
*/
pub fn calc_altitude(tjd_ut: f64, is_equal: bool, geo_lat: f64, geo_lng: f64, lng: f64, lat: f64) -> f64 {
  azalt(tjd_ut, is_equal, geo_lat, geo_lng, lng, lat).value
}

pub fn get_ayanamsha_value(jd: f64, key: &str) -> f64 {
  let aya_flag = Ayanamsha::from_key(key);
  get_ayanamsha(jd, aya_flag)
}

pub fn get_ayanamsha_values(jd: f64, keys: Vec<&str>) -> Vec<KeyNumValue> {
  let mut items: Vec<KeyNumValue> = Vec::new();
  for key in keys {
    let value = get_ayanamsha(jd, Ayanamsha::from_key(key));
    items.push(KeyNumValue::new(key, value));
  }
  items
}

pub fn get_all_ayanamsha_values(jd: f64) -> Vec<KeyNumValue> {
  let mut items: Vec<KeyNumValue> = Vec::new();
  let keys = all_ayanamsha_keys();
  for key in keys {
    let value = get_ayanamsha(jd, Ayanamsha::from_key(key));
    items.push(KeyNumValue::new(key, value));
  }
  items
}
