use serde::{Serialize, Deserialize};
use super::super::julian_date::*;
use super::super::traits::*;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GrahaPos {
  pub key: String,
  pub lng: f64,
  pub lat: f64,
  #[serde(rename="lngSpeed")]
  pub lng_speed: f64,
  #[serde(rename="latSpeed")]
  pub lat_speed: f64,
  #[serde(rename="rectAscension")]
  pub rect_ascension: f64,
  pub declination: f64,
}

impl GrahaPos {
  
  /** 
   * Default constructor for the ecliptic coordinate system without equatorial coordinates
   * The lng/lat speeds are ecliptic
   */
  pub fn new(key: &str, lng: f64, lat: f64, lng_speed: f64, lat_speed: f64) -> Self {
    GrahaPos { key: key.to_string(), lng: lng, lat, lng_speed: lng_speed, lat_speed: lat_speed, rect_ascension: 0f64, declination: 0f64 }
  }

  /** 
   * Default constructor for the equatorial coordinate system without ecliptic coordinates
   * The lng/lat speeds are equatorial
   */
  pub fn new_eq(key: &str, rect_ascension: f64, declination: f64, lng_speed: f64, lat_speed: f64) -> Self {
    GrahaPos { 
      key: key.to_string(),
      lng: 0f64, 
      lat: 0f64,
      lng_speed,
      lat_speed,
      rect_ascension,
      declination
    }
  }

  /** 
   * Default constructor for both the equatorial and ecliptic coordinate systems
   * The lng/lat speeds are ecliptic
   */
  pub fn new_both(key: &str, lng: f64, lat: f64, rect_ascension: f64, declination: f64, lng_speed: f64, lat_speed: f64) -> Self {
    GrahaPos { 
      key: key.to_string(),
      lng, 
      lat,
      lng_speed,
      lat_speed,
      rect_ascension,
      declination,
    }
  }

  /**
   * Default constructor for both the ecliptic coordinate systems without latitude speed
   */
  pub fn new_geo(key: &str, lng: f64, lat: f64, lng_speed: f64) -> Self {
    GrahaPos { key: key.to_string(), lng: lng, lat, lng_speed: lng_speed, lat_speed: 0f64, rect_ascension: 0f64, declination: 0f64 }
  }

  /**
   * Default constructor for a notional celestial object with only longitude and latitude using the ecliptic coordinate system
   * This may be used to elevate a true node to a full celestial object
   */
  pub fn fixed(key: &str, lng: f64, lat: f64) -> Self {
    GrahaPos { key: key.to_string(), lng: lng, lat: lat, lng_speed: 0f64, lat_speed: 0f64, rect_ascension: 0f64, declination: 0f64 }
  }

  /**
   * Default constructor for a notional celestial object with only longitude using the ecliptic coordinate system
   * This may be used to elevate the ascension to a full celestial object (graha)
   */
  pub fn basic(key: &str, lng: f64) -> Self {
    GrahaPos { key: key.to_string(), lng: lng, lat: 0f64, lng_speed: 0f64, lat_speed: 0f64, rect_ascension: 0f64, declination: 0f64 }
  }

}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GrahaPosItem {
  pub jd: f64,
  pub position: GrahaPos,
}

impl GrahaPosItem {
  pub fn new(jd: f64, pos: GrahaPos) -> GrahaPosItem {
    GrahaPosItem { jd, position: pos }
  }

}

impl ToISODateString for GrahaPosItem {

  fn iso_date_string(&self) -> String {
    julian_day_to_iso_datetime(self.jd)
  }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GrahaPosSet {
  pub jd: f64,
  pub bodies: Vec<GrahaPos>,
}

impl GrahaPosSet {
  pub fn new(jd: f64, bodies: Vec<GrahaPos>) -> GrahaPosSet {
    GrahaPosSet { jd, bodies }
  }
}

impl ToISODateString for GrahaPosSet {

  fn iso_date_string(&self) -> String {
    julian_day_to_iso_datetime(self.jd)
  }

}