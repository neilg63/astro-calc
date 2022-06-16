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
