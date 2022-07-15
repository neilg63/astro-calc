/* use super::models::{general::*, geo_pos::*, general::{LngLat}};
use super::settings::varga_values::*;
use super::utils::minmax::*; */
use super::julian_date::datetime_to_julian_day;

pub fn get_year_length(year_type: &str) -> f64 {
  let yt = match year_type {
    "sidereal" => 365.256366,
    "anomalistic" => 365.259636,
    _ => 365.242199
  };
  yt as f64
}

pub fn to_progression_jd(
  source_jd: f64,
  ref_jd: f64,
  year_type: &str,
) -> f64 {
  let age_in_days = ref_jd - source_jd;
  let projected_duration = age_in_days / get_year_length(year_type);
  source_jd + projected_duration
}

pub fn calc_progress_day_jds_by_year(source_jd: f64, start_year: u32, years: u16, per_year: u8) -> Vec<(f64, f64)> {
  let year_start_str = format!("{}-01-01T00:00:00", start_year);
  let start_jd = datetime_to_julian_day(year_start_str.as_str());
  let start_p2_jd = to_progression_jd(source_jd, start_jd, "tropical");
  let mut items: Vec<(f64, f64)> = Vec::new();
  let interval = 1f64 / per_year as f64;
  let interval_yr = get_year_length("tropical");
  let num_items = years as u32 * per_year as u32;
  for i in 0..num_items {
    let ref_jd = start_p2_jd + (interval * i as f64);
    let ref_year_jd = start_jd + (interval_yr * i as f64);
    items.push((ref_jd, ref_year_jd));
  }
  items
}

pub fn calc_opposite(lng: f64) -> f64 {
  (lng + 180f64) % 360f64
}

pub fn adjust_lng_by_body_key(key: &str, lng: f64) -> f64 {
  match key {
    "ke" => calc_opposite(lng),
    _ => lng,
  }
}

pub fn subtract_360(lng: f64, offset: f64) -> f64 {
  (lng + 360f64 - offset) % 360f64
}

/* 
pub fn match_house_num(lng: f64, houses: Vec<f64>) -> u8 {
  let len = houses.len();
  let min_val = if len > 0 { min_f64(houses.clone()) } else { 0f64 };
  let min_index = houses.clone().into_iter().position(|v| v == min_val).unwrap();
  let mut index: usize = 0;
  let matched_index = houses.iter().position(|deg| {
    let next_index = (index + 1) % len;
    let next = houses.get(next_index).unwrap();
    let end = if next < deg  { next + 360f64 } else { *next };
    let lng_plus = lng + 360f64;
    let ref_lng = if next < deg && next > &0f64 && lng_plus < end && min_index == next_index { lng_plus } else { lng };
    index += 1;
    ref_lng > *deg && ref_lng <= end
  });
  if let Some(m_index) = matched_index {
    m_index as u8 + 1u8
  } else {
    1u8
  }
}

pub fn map_sign_to_house(sign: u8, houses: Vec<f64>) -> u8 {
  let lng = (sign  * 30) as f64;
  match_house_num(lng, houses)
}

pub fn limit_value_to_range(num: f64, min: f64, max: f64) -> f64 {
  let span = max - min;
  let val = (num - min) % span;
  let ref_val = if val > 0f64 { val } else { span + val };
  let out_val = ref_val + min;
  if min < 0f64 && (val < 0f64 || num > max) { 0f64 - out_val } else { out_val }
}

pub fn calc_varga_value(lng: f64, num: u16) -> f64 {
  (lng * num as f64) % 360f64
}

pub fn calc_all_vargas(lng: f64) -> Vec<NumValue> {
  all_varga_items().into_iter().map(|v| {
    let value = calc_varga_value(lng, v.num);
    NumValue::new(v.num, value)
  }).collect()
}

pub fn calc_varga_set(lng: f64, num: u16, key: &str) -> NumValueKeySet {
  let values = calc_all_vargas(lng);
  NumValueKeySet::new(num, key, values)
}

pub fn calc_inclusive_distance(
  pos_1: u16,
  pos_2: u16,
  base: u16,
) -> u16 {
  ((pos_1 - pos_2 + base) % base) + 1
}

pub fn calc_inclusive_twelfths(pos_1: u16, pos_2: u16) -> u16 {
  calc_inclusive_distance(pos_1, pos_2, 12)
}
  

pub fn calc_inclusive_sign_positions(sign1: u8, sign2: u8) -> u8 {
  calc_inclusive_twelfths(sign2 as u16, sign1 as u16) as u8
}

pub fn calc_inclusive_nakshatras(pos_1: u8, pos_2: u8) -> u8 {
  calc_inclusive_twelfths(pos_1 as u16, pos_2 as u16) as u8
}

pub fn to_360(lng: f64) -> f64 {
  if lng >= 0f64 { lng + 180f64 }  else { 180f64 - lng }
}

pub fn from_360(lng: f64) -> f64 {
  if lng > 180f64 { lng - 180f64 } else { 0f64 - (180f64 - lng) }
}

pub fn median_lat(v1: f64, v2: f64) -> f64 {
  let offset = 90f64;
  return (v1 + offset + (v2 + offset)) / 2f64 - offset;
}

pub fn median_lng(v1: f64, v2: f64) -> f64 {
  let offset = 180f64;
  let full_circle = offset * 2f64;
  let d1 = to_360(v1);
  let d2 = to_360(v2);
  let reverse = (d2 - d1).abs() > offset;
  let is_west = if reverse { v1 + v2 > 0f64 } else { v1 + v2 < 0f64 };
  let res360 = ((d1 + d2) % full_circle) / 2f64;
  let res_a = from_360(res360) % offset;
  let res1 = if is_west { res_a } else { 0f64 - res_a };
  let res_b = offset - res1;
  let res2a = if is_west  { full_circle - res_b } else { res_b };
  let res2 = if is_west && res2a > 0f64  { 0f64 - res2a } else { res2a };
  let res2_is_between = to_360(res2) > d1 && to_360(res2) <= d2;
  let result = if reverse || res2_is_between  { res2 } else { res1 };
  result
}

pub fn median_lat_lng(coord1: GeoPos, coord2: GeoPos) -> LngLat {
  LngLat::new(
    median_lng(coord1.lng, coord2.lng),
    median_lat(coord1.lat, coord2.lat),
  )
}
 */

/* pub fn mid_point_to_surface(coord1: GeoPos, coord2: GeoPos) -> GeoPos {
  let c1 = geo_to_radians(coord1);
  let c2 = geo_to_radians(coord2);
  let bx = Math.cos(c2.lat) * Math.cos(c2.lng - c1.lng);
  let by = Math.cos(c2.lat) * Math.sin(c2.lng - c1.lng);
  let mid_lat = Math.atan2(
    Math.sin(c1.lat) + Math.sin(c2.lat),
    Math.sqrt((Math.cos(c1.lat) + bx) * (Math.cos(c1.lat) + bx) + by * by),
  );
  let mid_lng = c1.lng + Math.atan2(by, Math.cos(c1.lat) + bx);
  let mid_alt = (c1.alt + c2.alt) / 2f64;
  GeoPos::new( to_degrees(mid_lat), lng: to_degrees(mid_lng), mid_alt)
} */