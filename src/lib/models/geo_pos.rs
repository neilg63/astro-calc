use serde::{Serialize, Deserialize};

const default_altitude: f64 = 10f64;

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub struct GeoPos {
  pub lat: f64,
  pub lng: f64,
  pub alt: f64
}

impl GeoPos {
  pub fn new(lat: f64, lng: f64, alt: f64) -> Self {
    return GeoPos {
      lat: lat,
      lng: lng,
      alt: alt
    }
  }

  pub fn simple(lat: f64, lng: f64) -> Self {
    return GeoPos {
      lat: lat,
      lng: lng,
      alt: default_altitude
    }
  }

}