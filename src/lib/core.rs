use libswe_sys::sweconst::{Bodies, OptionalFlag};
use libswe_sys::swerust::{handler_swe03::*};
use super::settings::{ayanamshas::*, graha_values::*};
use super::models::{graha_pos::*};
use super::super::extensions::swe::{azalt};


pub fn calc_body_jd(jd: f64, sample_key: &str, sidereal: bool, topo: bool) -> GrahaPos {
  let mut combo = 0i32;
  if topo {
    if sidereal {
      combo = OptionalFlag::Speed as i32 | OptionalFlag::SideralPosition as i32 | OptionalFlag::TopocentricPosition as i32;
    } else {
      combo = OptionalFlag::Speed as i32 | OptionalFlag::TopocentricPosition as i32;
    }
  } else {
    if sidereal {
      combo = OptionalFlag::Speed as i32 | OptionalFlag::SideralPosition as i32;
    } else {
      combo = OptionalFlag::Speed as i32;
    }
  }
  let result = calc_ut(jd, Bodies::from_key(sample_key), combo);
  GrahaPos::new(sample_key, result.longitude, result.latitude, result.speed_longitude, result.speed_latitude)
}

pub fn calc_altitude(tjd_ut: f64, is_equal: bool, geo_lat: f64, geo_lng: f64, lng: f64, lat: f64) -> f64 {
  azalt(tjd_ut, is_equal, geo_lat, geo_lng, lng, lat).value
}
