use serde::{Serialize, Deserialize};
use libswe_sys::swerust::{handler_swe07::{PhenoUtResult}};
use super::{general::{LngLat, ToLngLat, LngLatKey, ToLngLatKey}};
use super::super::{julian_date::*, traits::*};
use super::super::super::extensions::swe::{AltitudeSet, azalt};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BodyPos {
  pub key: String,
  pub lng: f64,
  pub lat: f64,
  #[serde(rename="lngSpeed")]
  pub lng_speed: f64,
  #[serde(rename="latSpeed")]
  pub lat_speed: f64,
  pub mode: String,
}

impl BodyPos {
  pub fn new(key: &str, mode: &str, lng: f64, lat: f64, lng_speed: f64, lat_speed: f64) -> Self {
    BodyPos { key: key.to_string(), mode: mode.to_string(), lng: lng, lat: lat, lng_speed: lng_speed, lat_speed: lat_speed }
  }
}

impl ToLngLat for BodyPos {
  fn to_lng_lat(&self) -> LngLat {
    LngLat {
      lng: self.lng,
      lat: self.lat,
    }
  }
}

impl ToLngLatKey for BodyPos {
  fn to_lng_lat_key(&self) -> LngLatKey {
    LngLatKey {
      key: self.key.clone(),
      lng: self.lng,
      lat: self.lat,
    }
  }
}


#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PhenoResult {
  #[serde(rename="phaseAngle")]
  pub phase_angle: f64,
  #[serde(rename="phaseIlluminated")]
  pub phase_illuminated: f64,
  #[serde(rename="elongationOfPlanet")]
  pub elongation_of_planet: f64,
  #[serde(rename="apparentDiameterOfDisc")]
  pub apparent_diameter_of_disc: f64,
  #[serde(rename="apparentMagnitude")]
  pub apparent_magnitude: f64,
}

impl PhenoResult {
  pub fn new(phase_angle: f64, phase_illuminated: f64, elongation_of_planet: f64, apparent_diameter_of_disc: f64, apparent_magnitude: f64) -> PhenoResult {
    PhenoResult{ phase_angle: phase_angle, phase_illuminated, elongation_of_planet, apparent_diameter_of_disc,  apparent_magnitude }
  }

  pub fn new_from_result(result: PhenoUtResult) -> PhenoResult {
    PhenoResult{ 
      phase_angle: result.phase_angle,
      phase_illuminated: result.phase_illuminated,
      elongation_of_planet: result.elongation_of_planet,
      apparent_diameter_of_disc: result.apparent_dimaeter_of_disc,
      apparent_magnitude: result.apparent_magnitude
     }
  }
}


#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PhenoItem {
  pub key: String,
  #[serde(rename="phaseAngle")]
  pub phase_angle: f64,
  #[serde(rename="phaseIlluminated")]
  pub phase_illuminated: f64,
  #[serde(rename="elongationOfPlanet")]
  pub elongation_of_planet: f64,
  #[serde(rename="apparentDiameterOfDisc")]
  pub apparent_diameter_of_disc: f64,
  #[serde(rename="apparentMagnitude")]
  pub apparent_magnitude: f64,
}

impl PhenoItem {
  pub fn new(key: &str, phase_angle: f64, phase_illuminated: f64, elongation_of_planet: f64, apparent_diameter_of_disc: f64, apparent_magnitude: f64) -> PhenoItem {
    PhenoItem{ 
      key: key.to_string(),
      phase_angle,
      phase_illuminated,
      elongation_of_planet,
      apparent_diameter_of_disc, 
      apparent_magnitude
    }
  }

  pub fn new_from_result(key: &str, result: PhenoUtResult) -> PhenoItem {
    PhenoItem{ 
      key: key.to_string(),
      phase_angle: result.phase_angle,
      phase_illuminated: result.phase_illuminated,
      elongation_of_planet: result.elongation_of_planet,
      apparent_diameter_of_disc: result.apparent_dimaeter_of_disc,
      apparent_magnitude: result.apparent_magnitude
     }
  }
}

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
  #[serde(rename="lngSpeedEq")]
  pub lng_speed_eq: f64,
  #[serde(rename="latSpeedEq")]
  pub lat_speed_eq: f64,
  #[serde(skip_serializing_if = "Option::is_none")]
  pheno: Option<PhenoResult>,
  #[serde(skip_serializing_if = "Option::is_none")]
  altitude: Option<f64>,
  azimuth: Option<f64>
}

impl GrahaPos {
  
  /** 
   * Default constructor for the ecliptic coordinate system without equatorial coordinates
   * The lng/lat speeds are ecliptic
   */
  pub fn new(key: &str, lng: f64, lat: f64, lng_speed: f64, lat_speed: f64) -> Self {
    GrahaPos { 
      key: key.to_string(),
      lng,
      lat,
      lng_speed,
      lat_speed,
      rect_ascension: 0f64,
      declination: 0f64,
      lng_speed_eq: 0f64,
      lat_speed_eq: 0f64,
      pheno: None,
      altitude: None,
      azimuth: None
    }
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
      lng_speed: 0f64,
      lat_speed: 0f64,
      rect_ascension,
      declination,
      lng_speed_eq: lng_speed,
      lat_speed_eq: lat_speed,
      pheno: None,
      altitude: None,
      azimuth: None
    }
  }

  /** 
   * Default constructor for both the equatorial and ecliptic coordinate systems
   * The lng/lat speeds are ecliptic
   */
  pub fn new_both(key: &str, lng: f64, lat: f64, rect_ascension: f64, declination: f64, lng_speed: f64, lat_speed: f64, lng_speed_eq: f64, lat_speed_eq: f64) -> Self {
    GrahaPos { 
      key: key.to_string(),
      lng, 
      lat,
      lng_speed,
      lat_speed,
      rect_ascension,
      declination,
      lng_speed_eq,
      lat_speed_eq,
      pheno: None,
      altitude: None,
      azimuth: None
    }
  }

  pub fn new_extended(key: &str, lng: f64, lat: f64, rect_ascension: f64, declination: f64, lng_speed: f64, lat_speed: f64, lng_speed_eq: f64, lat_speed_eq: f64, pheno: Option<PhenoResult>, altitude: Option<f64>, azimuth: Option<f64>) -> Self {
    GrahaPos { 
      key: key.to_string(),
      lng, 
      lat,
      lng_speed,
      lat_speed,
      rect_ascension,
      declination,
      lng_speed_eq,
      lat_speed_eq,
      pheno,
      altitude,
      azimuth
    }
  }

  /**
   * Default constructor for both the ecliptic coordinate systems without latitude speed
   */
  pub fn new_geo(key: &str, lng: f64, lat: f64, lng_speed: f64) -> Self {
    GrahaPos { 
      key: key.to_string(),
      lng,
      lat,
      lng_speed,
      lat_speed: 0f64,
      rect_ascension: 0f64,
      declination: 0f64,
      lng_speed_eq: 0f64,
      lat_speed_eq: 0f64,
      pheno: None,
      altitude: None,
      azimuth: None
    }
  }

  /**
   * Default constructor for a notional celestial object with only longitude and latitude using the ecliptic coordinate system
   * This may be used to elevate a true node to a full celestial object
   */
  pub fn fixed(key: &str, lng: f64, lat: f64) -> Self {
    GrahaPos {
      key: key.to_string(),
      lng,
      lat,
      lng_speed: 0f64,
      lat_speed: 0f64,
      rect_ascension: 0f64,
      declination: 0f64,
      lng_speed_eq: 0f64,
      lat_speed_eq: 0f64,
      pheno: None,
      altitude: None,
      azimuth: None
    }
  }

  /**
   * Default constructor for a notional celestial object with only longitude using the ecliptic coordinate system
   * This may be used to elevate the ascension to a full celestial object (graha)
   */
  pub fn basic(key: &str, lng: f64) -> Self {
    GrahaPos {
      key: key.to_string(),
      lng, lat: 0f64,
      lng_speed: 0f64,
      lat_speed: 0f64,
      rect_ascension: 0f64,
      declination: 0f64,
      lng_speed_eq: 0f64,
      lat_speed_eq: 0f64,
      pheno: None,
      altitude: None,
      azimuth: None
    }
  }

  pub fn to_body(&self, mode: &str) -> BodyPos {
    let lng = match mode {
      "eq" => self.rect_ascension,
      _ => self.lng
    };
    let lat = match mode {
      "eq" => self.declination,
      _ => self.lat
    };
    let lng_speed = match mode {
      "eq" => self.lng_speed_eq,
      _ => self.lng_speed
    };
    let lat_speed = match mode {
      "eq" => self.lat_speed_eq,
      _ => self.lat_speed
    };
    BodyPos::new(self.key.as_str(), mode, lng, lat, lng_speed, lat_speed)
  }

}


impl ToLngLat for GrahaPos {
  fn to_lng_lat(&self) -> LngLat {
    LngLat {
      lng: self.lng,
      lat: self.lat,
    }
  }
}

impl ToLngLatKey for GrahaPos {
  fn to_lng_lat_key(&self) -> LngLatKey {
    LngLatKey {
      key: self.key.clone(),
      lng: self.lng,
      lat: self.lat,
    }
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

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum FlexiBodyPos {
  LngLat(Vec<LngLat>),
  LngLatKey(Vec<LngLatKey>),
  Simple(Vec<BodyPos>),
  Extended(Vec<GrahaPos>),
}