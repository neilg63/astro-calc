use ::serde::{Serialize, Deserialize};
use super::super::julian_date::{julian_day_to_iso_datetime};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct KeyNumValue {
  pub key: String,
  pub value: f64,
}

impl KeyNumValue {
  pub fn new(key: &str, value: f64) -> KeyNumValue {
    KeyNumValue { key: key.to_string(), value: value }
  }

  pub fn as_iso_string(&self) -> KeyStringValue {
    KeyStringValue { 
      key: self.key.clone(),
      value: julian_day_to_iso_datetime(self.value)
    }
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
pub struct KeyStringValueSet {
  pub key: String,
  pub items: Vec<KeyStringValue>,
}

impl KeyStringValueSet {
  pub fn new(key: &str, items: Vec<KeyStringValue>) -> KeyStringValueSet {
    KeyStringValueSet { key: key.to_string(), items }
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

  pub fn as_iso_strings(&self) -> KeyStringValueSet {
    KeyStringValueSet::new(self.key.as_str(), self.items.iter().map(|item| item.as_iso_string()).collect() )
  }

  pub fn as_flexi_values(&self, iso_mode: bool) -> KeyFlexiValueSet {
    KeyFlexiValueSet::new(self.key.as_str(), self.items.iter().filter(|item| item.value != 0f64).map(|item| match item.key.as_str() {
      "max" | "min" => FlexiValue::NumValue(item.to_owned()),
      _ => match iso_mode {
        true => FlexiValue::StringValue(item.as_iso_string()),
        _ => FlexiValue::NumValue(item.to_owned())
      }
    }).collect() )
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


#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum FlexiValueSet {
  NumValues(Vec<KeyNumValueSet>),
  StringValues(Vec<KeyStringValueSet>),
  FlexiValues(Vec<KeyFlexiValueSet>),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum FlexiValue {
  NumValue(KeyNumValue),
  StringValue(KeyStringValue),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct KeyFlexiValueSet {
  pub key: String,
  pub items: Vec<FlexiValue>,
}

impl KeyFlexiValueSet {
  pub fn new(key: &str, items: Vec<FlexiValue>) -> KeyFlexiValueSet {
    KeyFlexiValueSet { key: key.to_string(), items }
  }
}
