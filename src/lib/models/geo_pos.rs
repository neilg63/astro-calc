use serde::{Serialize, Deserialize};

const default_altitude: f64 = 10f64;

#[derive(Serialize, Deserialize, Debug)]
pub struct GeoPos {
  lat: f64,
  lng: f64,
  alt: f64
}

impl GeoPos {
  fn new(lat: f64, lng: f64, alt: f64) -> Self {
    return GeoPos {
      lat: lat,
      lng: lng,
      alt: alt
    }
  }

  fn simple(lat: f64, lng: f64) -> Self {
    return GeoPos {
      lat: lat,
      lng: lng,
      alt: default_altitude
    }
  }

}