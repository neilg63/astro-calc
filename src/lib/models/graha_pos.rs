#[derive(Debug, Clone)]
pub struct GrahaPos {
  pub key: String,
  pub lng: f64,
  pub lat: f64,
  pub lng_speed: f64,
  pub lat_speed: f64,
  pub rect_ascension: f64,
  pub declination: f64,
}

impl GrahaPos {
  
  pub fn new(key: &str, lng: f64, lat: f64, lng_speed: f64, lat_speed: f64) -> Self {
    GrahaPos { key: key.to_string(), lng: lng, lat, lng_speed: lng_speed, lat_speed: lat_speed, rect_ascension: 0f64, declination: 0f64 }
  }

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

  pub fn new_both(key: &str, lng: f64, lat: f64, rect_ascension: f64, declination: f64, lng_speed: f64, lat_speed: f64) -> Self {
    GrahaPos { 
      key: key.to_string(),
      lng, 
      lat,
      lng_speed,
      lat_speed,
      rect_ascension,
      declination
    }
  }

  pub fn new_geo(key: &str, lng: f64, lat: f64, lng_speed: f64) -> Self {
    GrahaPos { key: key.to_string(), lng: lng, lat, lng_speed: lng_speed, lat_speed: 0f64, rect_ascension: 0f64, declination: 0f64 }
  }

  pub fn fixed(key: &str, lng: f64, lat: f64) -> Self {
    GrahaPos { key: key.to_string(), lng: lng, lat: lat, lng_speed: 0f64, lat_speed: 0f64, rect_ascension: 0f64, declination: 0f64 }
  }

  pub fn basic(key: &str, lng: f64) -> Self {
    GrahaPos { key: key.to_string(), lng: lng, lat: 0f64, lng_speed: 0f64, lat_speed: 0f64, rect_ascension: 0f64, declination: 0f64 }
  }

}