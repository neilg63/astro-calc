use serde::{Deserialize,Serialize};
use std::collections::HashMap;
use super::lib::{models::{graha_pos::{BodyPos}, general::{LngLat}, date_info::DateInfo}, julian_date::{current_datetime_string}};
/* use actix_web::{web::{Query} };
use super::lib::{models::date_info::DateInfo, julian_date::{current_datetime_string}}; */


#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct KeyPos {
 pub key: String,
 pub lng: f64,
 pub lat: f64,
 #[serde(rename="lngSpeed")]
 pub lng_speed: f64,
}

impl KeyPos {
  pub fn to_body_pos(&self) -> BodyPos {
    BodyPos::new(self.key.as_str(), "ecl", self.lng, self.lat, self.lng_speed, 0f64)
  }
}

#[derive(Deserialize, Clone, Debug)]
pub struct PostOptions {
  pub dt: Option<String>, // primary UTC date string
  pub dtl: Option<String>, // primary date string in local time (requires offset)
  pub jd: Option<f64>, // primary jd as a float
  pub dt2: Option<String>, // secondary UTC date string 
  pub dtl2: Option<String>, // secondary date string in local time (requires offset)
  pub jd2: Option<f64>, // secondary jd as a float
  pub bodies: Option<Vec<String>>, // either a comma separated list of required 2-letter celestial body keys or body group keys
  pub positions: Option<Vec<KeyPos>>,
  pub lngs: Option<HashMap<String, f64>>,
  pub topo: Option<u8>, // 0 = geocentric, 1 topocentric, 2 both, default 0
  pub eq: Option<u8>, // 0 = ecliptic, 1 equatorial, 2 both, both 3 with altitude/azimuth, 4 with inline planetary phenomena
  pub ph: Option<u8>, // 0 = none (except via eq=4 in /chart-data), 1 = show pheno(nema) as separate array
  pub days: Option<u16>, // duration in days where applicable
  pub years: Option<u16>, // duration in years where applicable
  pub geo: Option<LngLat>, // lat,lng
  pub geo2: Option<LngLat>, // comma-separated lat,lng(,alt) numeric string
  pub iso: Option<bool>, // 0 show JD, 1 show ISO UTC
  pub tzs: Option<i32>, // offset in seconds from UTC
}

pub fn to_date_object_by_ref(params: &PostOptions) -> DateInfo {
  let jd = params.clone().jd.unwrap_or(0f64);
  if jd > 1_000_000f64 {
    DateInfo::new_from_jd(jd)
  } else {
    let dateref = params.clone().dt.unwrap_or(current_datetime_string());
    DateInfo::new(dateref.to_string().as_str())
  }
}