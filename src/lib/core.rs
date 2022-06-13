use libswe_sys::sweconst::{Bodies, OptionalFlag};
use libswe_sys::swerust::{handler_swe03::*};
use super::settings::{ayanamshas::*, graha_values::*};
use super::models::{graha_pos::*, geo_pos::*};
use super::super::extensions::swe::{azalt, set_topo};


pub fn calc_body_jd(jd: f64, sample_key: &str, sidereal: bool, topo: bool) -> GrahaPos {
  let combo: i32;
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

pub fn calc_body_jd_geo(jd: f64, sample_key: &str, sidereal: bool) -> GrahaPos {
  calc_body_jd(jd, sample_key, sidereal, false)
}

pub fn calc_body_jd_topo(jd: f64, sample_key: &str, geo: GeoPos, sidereal: bool) -> GrahaPos {
  set_topo(geo.lat, geo.lng, geo.alt);
  calc_body_jd(jd, sample_key, sidereal, true)
}


pub fn calc_altitude(tjd_ut: f64, is_equal: bool, geo_lat: f64, geo_lng: f64, lng: f64, lat: f64) -> f64 {
  azalt(tjd_ut, is_equal, geo_lat, geo_lng, lng, lat).value
}
