use std::os::raw::{c_char, c_double, c_int};
use serde::{Serialize, Deserialize};
use libswe_sys::sweconst::{Bodies};
use libswe_sys::swerust::{handler_swe14::*};
use super::super::lib::settings::ayanamshas::*;

pub enum TransitionParams {
  Rise = 1,
  Set = 2,
  Mc = 4,
  Ic = 8,
  Center = 256,
  Bottom = 8192,
  BitNoRefraction = 512,
  BitGeoctrNoEclLat = 128
}

enum BodyAltitudes {
  EquToHor =1,
  EclToHor = 0,
}

impl TransitionParams {
  pub fn center_disc_rising() -> i32 {
    TransitionParams::Center as i32 | TransitionParams::BitNoRefraction as i32 | TransitionParams::BitGeoctrNoEclLat as i32
  }

  pub fn center_disc_rising_rise() -> i32 {
    TransitionParams::center_disc_rising() + TransitionParams::Rise as i32
  }

  pub fn center_disc_rising_set() -> i32 {
    TransitionParams::center_disc_rising() + TransitionParams::Set as i32
  }

  pub fn mc() -> i32 {
    TransitionParams::Mc as i32
  }

  pub fn ic() -> i32 {
    TransitionParams::Ic as i32
  }
}

#[link(name = "swe")]
extern "C" {
  /*
  jd: number;
planetNum: number;
iflag: number;
transType: number;
longitude: number;
latitude: number;
altitude: number;
pressure: number;
temperature: number;
  */
  
  pub fn swe_rise_trans(
      tjd_ut: c_double,
      ipl: c_int,
      starname: *mut [c_char; 0],
      epheflag: c_int,
      rsmi: c_int,
      geopos: *mut [c_double; 3],
      atpress: c_double,
      attemp: c_double,
      tret: *mut c_double,
      serr: *mut c_char
  ) -> c_double;

  /*
   double tjd_ut,
    int32  calc_flag,
    double *geopos,
    double atpress,
    double attemp,
    double *xin, 
    double *xaz) 
  */

  pub fn swe_azalt(
      tjd_ut: c_double,
      iflag: c_int,
      geopos: *mut [c_double; 3],
      atpress: c_double,
      attemp: c_double,
      xin: *mut [c_double; 2],
      xaz: *mut [c_double; 3]
  );


  pub fn swe_get_ayanamsa_ex_ut(
      jd: c_double,
      iflag: c_int,
      daya: *mut c_double,
      serr: *mut c_char
  ) -> c_double;

  // swe_set_topo(double geolon, double geolat, double geoalt);

  pub fn swe_set_topo(
    lng: c_double,
    lat: c_double,
    alt: c_double
  );

  // swe_set_sid_mode(sidModeNum, 0, 0);
  pub fn swe_set_sid_mode(sid_mode: i32, t9: f64, ayan_t0: f64);

}

pub fn rise_trans(tjd_ut: f64, ipl: Bodies, lat: f64, lng: f64, iflag: i32) -> f64 {
  let mut xx: [f64; 6] = [0.0; 6];
  let mut serr = [0; 255];
  let geopos = &mut [lng, lat, 0f64];
  let star_ref = &mut [];
  let result = unsafe {
      let p_xx = xx.as_mut_ptr();
      let p_serr = serr.as_mut_ptr();
      let status = swe_rise_trans(
          tjd_ut,
          ipl as i32,
          star_ref,
          0,
          iflag,
          geopos,
          0f64,
          0f64,
          p_xx,
          p_serr,
      );
      status
  };
  result
}

/*
  Wrapper for swe_azalt.
  tjd_jd: Julian Day,
  is_equal: if true 
*/
pub fn azalt(tjd_ut: f64, is_equal: bool, geo_lat: f64, geo_lng: f64, lng: f64, lat: f64) -> AltitudeSet {
  let iflag = if is_equal { BodyAltitudes::EquToHor } else { BodyAltitudes::EclToHor } as i32;
  let geopos = &mut [geo_lng, geo_lat, 0f64];
  let result = unsafe {
      let p_xin = &mut [lng, lat];
      let p_xaz = &mut [0f64, 0f64, 0f64];
      swe_azalt(
          tjd_ut,
          iflag,
          geopos,
          0f64,
          0f64,
          p_xin,
          p_xaz,
      );
      *p_xaz
  };
  AltitudeSet{
      azimuth: result[0],
      value: result[1],
      apparent: result[2],
  }
}

pub fn get_ayanamsha(tjd_ut: f64, mode: Ayanamsha) -> f64 {
  let mut daya: [f64; 1] = [0.0; 1];
  let mut serr = [0; 255];
  let result = unsafe {
      let p_daya = daya.as_mut_ptr();
      let p_serr = serr.as_mut_ptr();
      let status = swe_get_ayanamsa_ex_ut(
          tjd_ut,
          mode as i32,
          p_daya,
          p_serr
      );
      status
  };
  result
}

pub fn set_topo(lat: f64, lng: f64, alt: f64) {
  unsafe {
    swe_set_topo(lng, lat, alt);
  }
}

pub fn set_sid_mode(iflag: i32) {
  unsafe {
    swe_set_sid_mode(iflag, 0f64, 0f64);
  }
}

pub fn next_rise(tjd_ut: f64, ipl: Bodies, lat: f64, lng: f64) -> f64 {
  rise_trans(tjd_ut, ipl, lat, lng, TransitionParams::center_disc_rising_rise())
}

pub fn next_set(tjd_ut: f64, ipl: Bodies, lat: f64, lng: f64) -> f64 {
  rise_trans(tjd_ut, ipl, lat, lng, TransitionParams::center_disc_rising_set())
}

pub fn next_mc(tjd_ut: f64, ipl: Bodies, lat: f64, lng: f64) -> f64 {
  rise_trans(tjd_ut, ipl, lat, lng, TransitionParams::mc())
}

pub fn next_mc_q(tjd_ut: f64, ipl: Bodies, lat: f64, lng: f64, rise_jd: f64) -> f64 {
let set_jd = rise_trans(tjd_ut, ipl, lat, lng, TransitionParams::Set as i32);
((set_jd - rise_jd) / 2f64) + rise_jd
}

pub fn next_ic(tjd_ut: f64, ipl: Bodies, lat: f64, lng: f64) -> f64 {
  rise_trans(tjd_ut, ipl, lat, lng, TransitionParams::ic())
}

pub fn next_ic_q(tjd_ut: f64, ipl: Bodies, lat: f64, lng: f64, set_jd: f64) -> f64 {
  let next_rise_jd = rise_trans(tjd_ut + 1f64, ipl, lat, lng, TransitionParams::Rise as i32);
  ((next_rise_jd - set_jd) / 2f64) + set_jd
}

pub fn start_jd_geo(jd: f64, lng: f64) -> f64 {
  let offset = (0f64 - lng / 15f64) / 24f64;
  let jd_progress = jd % 0f64;
  let adjusted_progress = jd_progress + offset;
  let start_offset = if adjusted_progress >= 0.5 { 0.5f64 } else { -0.5f64 };
  let start = jd.floor() + start_offset;
  start + offset
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TransitSet {
  pub prev_set: f64,
  pub rise: f64,
  pub mc: f64,
  pub set: f64,
  pub ic: f64,
  pub next_rise: f64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AltitudeSet {
  pub azimuth: f64,
  pub value: f64,
  pub apparent: f64,
}


pub fn calc_transit_set(jd: f64, ipl: Bodies, lat: f64, lng: f64) -> TransitSet {
  let ref_jd = start_jd_geo(jd, lng);
  let prev_set = next_set(ref_jd - 1f64, ipl, lat, lng);
  let rise = next_rise(ref_jd, ipl, lat, lng);
  let set = next_set(ref_jd, ipl, lat, lng);
  let next_rise = next_rise(ref_jd + 1f64, ipl, lat, lng);

  //let mc = next_mc_q(ref_jd, ipl, lat, lng, rise);
  let mc = next_mc(ref_jd, ipl, lat, lng);
  //let mc = 0f64;
  //let ic = 0f64;
  //let ic = next_ic_q(ref_jd, ipl, lat, lng, set);
  let ic = next_ic(ref_jd, ipl, lat, lng);
  TransitSet { 
    prev_set: prev_set,
    rise: rise,
    mc: mc,
    set: set,
    ic: ic,
    next_rise: next_rise
  }
}


#[derive(Serialize, Deserialize, Debug)]
pub struct AscMc {
  ascendant: f64,
  mc: f64,
  armc: f64,
  vertex: f64,
  equasc: f64,		// "equatorial ascendant" *
  coasc1: f64,		// "co-ascendant" (W. Koch) *
  coasc2: f64,		// "co-ascendant" (M. Munkasey) *
  polasc: f64,
}

impl AscMc {
  pub fn new(points: [f64; 10]) -> AscMc {
      AscMc {
        ascendant: points[0],
        mc: points[1],
        armc: points[2],
        vertex: points[3],
        equasc: points[4],
        coasc1: points[5],
        coasc2: points[6],
        polasc: points[7],
      }
  }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct HouseData {
  pub jd: f64,
  pub lat: f64,
  pub lng: f64,
  pub system: char,
  pub houses: Vec<f64>,
  pub points: AscMc,
}

impl HouseData {

  pub fn new(jd: f64, lat: f64, lng: f64, system: char) -> HouseData {
    let hd = houses(jd, lat, lng, system);
      HouseData {
        jd: jd,
        lng: lng,
        lat: lat,
        system: system,
        houses: hd.cusps,
        points: AscMc::new(hd.ascmc)
    }
  }
}

pub fn get_house_data(jd: f64, lat: f64, lng: f64, system: char) -> HouseData {
  HouseData::new(jd, lat, lng, system)
}

pub fn get_ascendant(jd: f64, lat: f64, lng: f64) -> f64 {
  let hd = houses(jd, lat, lng, 'W');
  hd.ascmc[0]
}
