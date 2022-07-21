use super::super::models::geo_pos::{GeoPos};

pub fn to_str_refs(strings: &Vec<String>) -> Vec<&str> {
  let strs: Vec<&str> = strings.iter().map(|s| s.as_ref()).collect();
  strs
}

pub fn body_keys_str_to_keys(key_string: String) -> Vec<String> {
  key_string.split(",").into_iter().filter(|p| p.len() == 2).map(|p| p.to_string()).collect()
}

pub fn body_keys_str_to_keys_or(key_string: String, default_keys: Vec<&str>) -> Vec<String> {
  let keys: Vec<String> = body_keys_str_to_keys(key_string);
  if keys.len() > 0 { keys.into_iter().filter(|k| k.len() == 2 && !k.contains("as")).collect() } else { default_keys.into_iter().map(|p| p.to_string() ).collect() }
}

pub fn loc_string_to_geo(loc: &str) -> Option<GeoPos> {
  let parts: Vec<f64> = loc.split(",").into_iter().map(|p| p.parse::<f64>()).filter(|p| match p { Ok(_n) => true, _ => false } ).map(|p| p.unwrap()).collect();
  if parts.len() >= 2 {
    let alt = if parts.len() > 2 { parts[2] } else { 0f64 };
    Some(GeoPos::new(parts[0], parts[1], alt))
  } else {
    None
  }
}