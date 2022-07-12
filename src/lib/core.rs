use math::round::{floor};
use libswe_sys::sweconst::{Bodies, OptionalFlag};
use libswe_sys::swerust::{handler_swe03::*};
use super::{settings::{ayanamshas::*},traits::*, math_funcs::{calc_progress_day_jds_by_year, adjust_lng_by_body_key}, math_funcs::{subtract_360}, transitions::{get_pheno_result}, transposed_transitions::{calc_transitions_from_source_refs_minmax}};
use super::models::{graha_pos::*, geo_pos::*, general::*, houses::{calc_ascendant}};
use super::super::extensions::swe::{azalt, set_topo, set_sid_mode, get_ayanamsha};
use std::collections::{HashMap};

pub fn calc_body_jd(jd: f64, key: &str, sidereal: bool, topo: bool) -> GrahaPos {
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
  let result = calc_ut(jd, Bodies::from_key(key), combo);
  let lng = adjust_lng_by_body_key(key, result.longitude);
  GrahaPos::new(key, lng, result.latitude, result.speed_longitude, result.speed_latitude)
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
  let lng = adjust_lng_by_body_key(key, result.longitude);
  GrahaPos::new_eq(key, result.longitude, result.latitude, lng, result.speed_latitude)
}

pub fn calc_body_dual_jd(jd: f64, key: &str, topo: bool, show_pheno: bool, geo_opt: Option<GeoPos>) -> GrahaPos {
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
  let pheno = if show_pheno { Some(get_pheno_result(jd, key, 0i32)) } else { None };
  let lng = adjust_lng_by_body_key(key, result_geo.longitude);
  let ra = adjust_lng_by_body_key(key, result.longitude);
  let altitude_set = match geo_opt {
    Some(geo) => Some(azalt(jd, true, geo.lat, geo.lng, result.longitude, result.latitude)),
    _ => None,
  };
  let altitude = match altitude_set {
    Some(a_set) => Some(a_set.value),
    None => None
  };
  let azimuth = match altitude_set {
    Some(a_set) => Some(a_set.azimuth),
    None => None
  };
  GrahaPos::new_extended(key, lng, result_geo.latitude,  ra, result.latitude, result_geo.speed_longitude, result_geo.speed_latitude,  result.speed_longitude, result.speed_latitude, pheno, altitude, azimuth)
}

pub fn calc_body_dual_jd_geo(jd: f64, key: &str, show_pheno: bool) -> GrahaPos {
  calc_body_dual_jd(jd, key, false, show_pheno, None)
}

pub fn calc_body_dual_jd_topo(jd: f64, key: &str, geo: GeoPos, show_pheno: bool) -> GrahaPos {
  set_topo(geo.lat, geo.lng, geo.alt);
  calc_body_dual_jd(jd, key, true, show_pheno, Some(geo))
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
pub fn calc_bodies_positions_jd(jd_start: f64, keys: Vec<&str>, days: u16, num_per_day: f64, geo: Option<GeoPos>, eq: bool) -> Vec<GrahaPosSet> {
  let mut items: Vec<GrahaPosSet> = Vec::new();
  let max_f64 = floor(days as f64 * num_per_day, 0);
  let max = max_f64 as i32;
  let increment = 1f64 / num_per_day;
  let topo = match geo {
    None => false,
    _ => true,
  };
  for i in 0..max {
    let curr_jd = jd_start + (i as f64 * increment);
    let mut bodies: Vec<GrahaPos> = Vec::new();
    for key in &keys {
      let graha_pos = match eq {
        true => match topo {
          true => calc_body_eq_jd_topo(curr_jd, key, geo.unwrap()),
          _ => calc_body_eq_jd(curr_jd, key, false),
        },
        _ => match topo {
          true => calc_body_jd_topo(curr_jd, key, geo.unwrap()),
          _ => calc_body_jd_geo(curr_jd, key),
        }
      };
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
pub fn get_bodies_dual_geo(jd: f64, keys: Vec<&str>, show_pheno: bool) -> Vec<GrahaPos> {
  let mut bodies: Vec<GrahaPos> = Vec::new();
  for key in keys {
    let result = calc_body_dual_jd_geo(jd, key, show_pheno);
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

pub fn get_bodies_eq_geo(jd: f64, keys: Vec<&str>) -> Vec<GrahaPos> {
  let mut bodies: Vec<GrahaPos> = Vec::new();
  for key in keys {
    let result = calc_body_eq_jd(jd, key, false);
    bodies.push(result);
  }
  bodies
}

pub fn get_bodies_eq_topo(jd: f64, keys: Vec<&str>, geo: GeoPos) -> Vec<GrahaPos> {
  let mut bodies: Vec<GrahaPos> = Vec::new();
  for key in keys {
    let result = calc_body_eq_jd_topo(jd, key, geo);
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

pub fn get_bodies_p2(jd: f64, keys: Vec<String>, start_year: u32, num_years: u16, per_year: u8) -> Vec<ProgressItemSet> {
  let mut items: Vec<ProgressItemSet> = Vec::new();
  let jd_pairs = calc_progress_day_jds_by_year(jd, start_year, num_years, per_year);
  for pair in jd_pairs {
    let (ref_pd, ref_jd) = pair;
    let ayanamsha = get_ayanamsha_value(ref_pd, "true_citra");
    let mut body_items: Vec<KeyNumValue> = Vec::new();
    for key in keys.clone() {
      let result = calc_body_jd_geo(jd, key.as_str());
      body_items.push(KeyNumValue::new(key.as_str(), result.lng));
    }
    items.push(ProgressItemSet::new(ref_pd, ref_jd, body_items, ayanamsha));
  }
  items
}

pub fn get_body_longitudes(jd: f64, geo: GeoPos, mode: &str, equatorial: bool, aya_offset: f64, keys: Vec<&str>) -> HashMap<String, f64> {
  let mut items: HashMap<String, f64> = HashMap::new();
  let bodies = match equatorial {
    true => match mode {
      "topo" => get_bodies_eq_topo(jd, keys, geo),
      _ => get_bodies_eq_geo(jd, keys),
    },
    _ => match mode {
      "topo" => get_bodies_ecl_topo(jd, keys, geo),
      _ => get_bodies_ecl_geo(jd, keys),
    }
  };
  items.insert("as".to_string(), subtract_360(calc_ascendant(jd, geo), aya_offset));
  for body in bodies {
    let lng = if equatorial { body.rect_ascension } else { body.lng };
    items.insert(body.key, subtract_360(lng, aya_offset));
  }
  items
}


pub fn get_body_longitudes_geo(jd: f64, geo: GeoPos, aya_offset: f64, keys: Vec<&str>) -> HashMap<String, f64> {
  get_body_longitudes(jd, geo, "geo", false, aya_offset, keys)
}

pub fn get_body_longitudes_topo(jd: f64, geo: GeoPos, aya_offset: f64, keys: Vec<&str>) -> HashMap<String, f64> {
  get_body_longitudes(jd, geo, "topo", false, aya_offset, keys)
}

pub fn get_body_longitudes_eq_geo(jd: f64, geo: GeoPos, aya_offset: f64, keys: Vec<&str>) -> HashMap<String, f64> {
  get_body_longitudes(jd, geo, "geo", true, aya_offset, keys)
}

pub fn get_body_longitudes_eq_topo(jd: f64, geo: GeoPos, aya_offset: f64, keys: Vec<&str>) -> HashMap<String, f64> {
  get_body_longitudes(jd, geo, "topo", true, aya_offset, keys)
}

pub fn get_bodies_dual_topo(jd: f64, keys: Vec<&str>, geo: GeoPos, show_pheno: bool) -> Vec<GrahaPos> {
  let mut bodies: Vec<GrahaPos> = Vec::new();
  for key in keys {
    let result = calc_body_dual_jd_topo(jd, key, geo, show_pheno);
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

/*
* Match the projected altitude of any celestial object
*/
pub fn calc_altitude_object(tjd_ut: f64, is_equal: bool, geo_lat: f64, geo_lng: f64, key: &str) -> f64 {
  let pos = match is_equal {
    true => calc_body_eq_jd_topo(tjd_ut, key, GeoPos::simple(geo_lat, geo_lng)),
    _ => calc_body_jd_topo(tjd_ut, key, GeoPos::simple(geo_lat, geo_lng))
  };
  calc_altitude(tjd_ut, is_equal, geo_lat, geo_lng, pos.lng, pos.lat)
}



pub fn calc_next_prev_horizon(jd: f64, geo_lat: f64, geo_lng: f64, key: &str, down: bool, next: bool) -> f64 {
  let unit = if next { 1f64 } else { -1f64 };
  let mut alt = calc_altitude_object(jd, false, geo_lat, geo_lng, key);
  let mut days: u16 = 1;
  let mut day_jd = 0f64;
  while ((down && alt < 0f64) || (!down && alt > 0f64)) && days < 184 {
    let ref_jd = jd + (unit * days as f64);
    alt = calc_altitude_object(ref_jd, false, geo_lat, geo_lng, key);
    days += 1;
    day_jd = ref_jd.clone();
  }
  if day_jd > 100f64 { 
    let geo = GeoPos::simple(geo_lat, geo_lng);
    let mut base = calc_transitions_from_source_refs_minmax(day_jd, key, geo);
    
    let mut new_day_jd = if (down && !next) || (!down && next) { base.set } else { base.rise };
    
    if new_day_jd < 100f64 {
      let day_down = base.min < 0f64 && base.max < 0f64;
      let next_jd = if (day_down && next) || (!day_down && !next)  { day_jd + 1f64 } else { day_jd - 1f64 };
      base = calc_transitions_from_source_refs_minmax(next_jd, key, geo);
      new_day_jd = if (down && !next) || (!down && next) { base.set } else { base.rise };
      if new_day_jd < 100f64 {
        let next_jd = if (day_down && next) || (!day_down && !next)  { day_jd - 1f64 } else { day_jd + 1f64 };
        base = calc_transitions_from_source_refs_minmax(next_jd, key, geo);
        new_day_jd = if (down && !next) || (!down && next) { base.set } else { base.rise };
      }
    }
    day_jd = new_day_jd;
  }
  day_jd
}

/*
* reconstructed from Lahiri by calculating proportional differences over 200 years. Native C implementation may be bug-prone
* on some platforms.
*/
pub fn calc_true_citra(jd: f64) -> f64 {
  let jd1 = 2422324.5f64;
  let p1 = 0.9992925739019888f64;
  let jd2 = 2458849.5f64;
  let p2 = 0.99928174751934f64;
  let jd3 = 2495373.5f64;
  let p3 = 0.9992687765534588f64;
  let diff_jd2 = jd - jd2;
  let before2020 = diff_jd2 < 0f64;
  let dist = if before2020 { (0f64 - diff_jd2) / (jd2 - jd1) } else { diff_jd2 / (jd3 - jd2) };
  let diff_p = if before2020 {  p2 - p1 } else { p3 - p2 };
  let multiple = if before2020 { p2 - (diff_p * dist) } else { p2 + (diff_p * dist) };
  get_ayanamsha_value_raw(jd, "lahiri") * multiple
}

pub fn get_ayanamsha_value_raw(jd: f64, key: &str) -> f64 {
  let aya_flag = Ayanamsha::from_key(key);
  get_ayanamsha(jd, aya_flag)
}

pub fn get_ayanamsha_value(jd: f64, key: &str) -> f64 {
  let aya_flag = Ayanamsha::from_key(key);
  match aya_flag {
    Ayanamsha::Tropical => 0f64,
    Ayanamsha::TrueCitra => calc_true_citra(jd),
    _ => get_ayanamsha(jd, aya_flag)
  }
}

pub fn get_ayanamsha_values(jd: f64, keys: Vec<&str>) -> Vec<KeyNumValue> {
  let mut items: Vec<KeyNumValue> = Vec::new();
  for key in keys {
    let value = get_ayanamsha_value(jd, key);
    items.push(KeyNumValue::new(match_ayanamsha_key(key).as_str(), value));
  }
  items
}

pub fn get_all_ayanamsha_values(jd: f64) -> Vec<KeyNumValue> {
  let mut items: Vec<KeyNumValue> = Vec::new();
  let keys = all_ayanamsha_keys();
  for key in keys {
    let value = get_ayanamsha_value(jd, key);
    items.push(KeyNumValue::new(key, value));
  }
  items
}
