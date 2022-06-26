use ::serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct KeyNumValue {
  pub key: String,
  pub value: f64,
}

impl KeyNumValue {
  pub fn new(key: &str, value: f64) -> KeyNumValue {
    KeyNumValue { key: key.to_string(), value: value }
  }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct KeyStringValue {
  pub key: String,
  pub value: String,
}

impl KeyStringValue {
  pub fn new(key: &str, value: &str) -> KeyStringValue {
    KeyStringValue { key: key.to_string(), value: value.to_string() }
  }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct KeyNumValueSet {
  pub key: String,
  pub items: Vec<KeyNumValue>,
}

impl KeyNumValueSet {
  pub fn new(key: &str, items: Vec<KeyNumValue>) -> KeyNumValueSet {
    KeyNumValueSet { key: key.to_string(), items }
  }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NumValue {
  pub num: u16,
  pub value: f64,
}

impl NumValue {
  pub fn new(num: u16, value: f64) -> NumValue {
    NumValue { num, value }
  }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NumValueKeySet {
  pub num: u16,
  pub key: String,
  pub values: Vec<NumValue>,
}

impl NumValueKeySet {
  pub fn new(num: u16, key: &str, values: Vec<NumValue>) -> NumValueKeySet {
    NumValueKeySet { num, key: key.to_string(), values }
  }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProgressItemSet {
  pd: f64,
  jd: f64,
  bodies : Vec<KeyNumValue>,
  ayanamsha: f64,
}

impl ProgressItemSet {
  pub fn new(pd: f64, jd: f64, bodies: Vec<KeyNumValue>, ayanamsha: f64) -> ProgressItemSet {
    ProgressItemSet { pd, jd, bodies, ayanamsha }
  }
}

/**
 * Used for celestial objects
 */

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub struct LngLat {
  pub lng: f64,
  pub lat: f64,
}

impl LngLat {
  pub fn new(lng: f64, lat: f64) -> LngLat {
    LngLat { lng, lat }
  }
  pub fn empty() -> LngLat {
    LngLat { lng: -1f64, lat: -1f64 }
  }
}

pub trait ToLngLat {
  fn to_lng_lat(&self) -> LngLat;
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LngLatKey {
  pub lng: f64,
  pub lat: f64,
  pub key: String,
}

impl LngLatKey {
  pub fn new(key: &str, lng: f64, lat: f64) -> LngLatKey {
    LngLatKey { key: key.to_string(), lng, lat }
  }
}

pub trait ToLngLatKey {
  fn to_lng_lat_key(&self) -> LngLatKey;
}
