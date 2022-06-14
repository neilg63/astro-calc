use libswe_sys::sweconst::{Bodies, OptionalFlag};
use libswe_sys::swerust::{handler_swe03::*};
use super::settings::{ayanamshas::*, graha_values::*};
use super::traits::*;
use super::models::{graha_pos::*, geo_pos::*};
use super::super::extensions::swe::{azalt, set_topo, set_sid_mode};


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
  println!("calc_ut result: {:?}", result);
  GrahaPos::new(sample_key, result.longitude, result.latitude, result.speed_longitude, result.speed_latitude)
}

/*
 Get tropical geocentric coordinates
*/
pub fn calc_body_jd_geo(jd: f64, sample_key: &str) -> GrahaPos {
  calc_body_jd(jd, sample_key, false, false)
}

/*
 Get sidereal geocentric coordinates with an ayanamsha key
*/
pub fn calc_body_jd_geo_sidereal(jd: f64, sample_key: &str, aya_key: &str) -> GrahaPos {
  set_sid_mode(Ayanamsha::from_key(aya_key).as_i32());
  calc_body_jd(jd, sample_key, true, false)
}

/*
 Get tropical topocentric coordinates with geo-coordinates
*/
pub fn calc_body_jd_topo(jd: f64, sample_key: &str, geo: GeoPos) -> GrahaPos {
  set_topo(geo.lat, geo.lng, geo.alt);
  calc_body_jd(jd, sample_key, false, true)
}

/*
 Get sidereal topocentric coordinates with geo-coordinates and an ayanamsha key
*/
pub fn calc_body_jd_topo_sidereal(jd: f64, sample_key: &str, geo: GeoPos, aya_key: &str) -> GrahaPos {
  set_topo(geo.lat, geo.lng, geo.alt);
  set_sid_mode(Ayanamsha::from_key(aya_key).as_i32());
  calc_body_jd(jd, sample_key, false, true)
}

/*
* Match the projected altitude of any celestial object
*/
pub fn calc_altitude(tjd_ut: f64, is_equal: bool, geo_lat: f64, geo_lng: f64, lng: f64, lat: f64) -> f64 {
  azalt(tjd_ut, is_equal, geo_lat, geo_lng, lng, lat).value
}
