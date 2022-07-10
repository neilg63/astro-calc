use chrono::*;
use ::serde::{Serialize, Deserialize};
use super::{geo_pos::*, general::*};
use super::super::{julian_date::{julian_day_to_datetime}};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Variant {
  num: u8, // ayanamsha ref number
  sign: u8,
  house: u8,
  nakshatra: u8,
  relationship: String,
  #[serde(rename="charaKaraka")]
  chara_karaka: u8,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct VariantSet {
  num: u8,
  items: Vec<KeyNumValue>,
}

/**
 * Used for celestial objects
 */

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HouseSystem {
  system: char,
  values: Vec<f64>
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Transition {
  mode: String,
  jd: f64,

}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Graha {
  key: String,
  lng: f64,
  lat: f64,
  topo: LngLat,
  lng_speed: f64,
  declination: f64,
  variants: Vec<Variant>,
  transitions: Vec<Transition>,
}

impl Graha {

  pub fn new_geo(key: &str, lng: f64, lat: f64, lng_speed: f64, declination: f64) -> Graha {
    Graha{
      key: key.to_string(),
      lng: lng,
      lat: lat,
      lng_speed: lng_speed,
      declination: declination,
      variants: vec![],
      transitions: vec![],
      topo: LngLat::empty()
    }
  }

  pub fn simple(key: &str, lng: f64) -> Graha {
    Graha{
      key: key.to_string(), lng, lat: 0f64, lng_speed: 0f64, declination: 0f64, variants: Vec::new(), transitions: Vec::new(), topo: LngLat::empty()
    }
  }

  pub fn adjusted(gr: Graha, aya_val: f64) -> Graha {
    let lng = (gr.lng + aya_val + 360f64) % 360f64;
    Graha{
      key: gr.key,
      lng: lng,
      lat: gr.lat,
      lng_speed: gr.lng_speed,
      declination: gr.declination,
      variants: gr.variants,
      transitions: gr.transitions,
      topo: gr.topo
    }
  }

  pub fn set_ayanamsha(&mut self, value: f64) {
    self.lng = (self.lng + value + 360f64) % 360f64;
  }

  pub fn set_topo(&mut self, lng: f64, lat: f64) {
    self.topo = LngLat::new(lng, lat);
  }

  pub fn set_transitions(&mut self, transitions: Vec<Transition>) {
    self.transitions = transitions;
  }

  pub fn add_variant_set(&mut self, variant: Variant) {
    self.variants.push(variant);
  }

}

#[derive(Serialize, Deserialize, Debug)]
pub struct Rashi {
  #[serde(rename="houseNum")]
  house_num: u8,
  sign: u8,
  #[serde(rename="lordInHouse")]
  lord_in_house: u8,
  #[serde(rename="arudhaInHouse")]
  arudha_in_house: u8,
  #[serde(rename="arudhaInSign")]
  arudha_in_sign: u8,
  #[serde(rename="arudhaLord")]
  arudha_lord: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RashiSet {
  num: u8,
  items: Vec<Rashi>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Placename {
  name: String,
  #[serde(rename="fullName")]
  full_name: String,
  r#type: String,
  geo: GeoPos,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Subject {
  name: String,
  notes: String,
  r#type: String,
  gender: String,
  #[serde(rename="eventType")]
  event_type: String,
  #[serde(rename="roddenValue")]
  rodden_value: i32,
  #[serde(rename="altNames")]
  alt_names: Vec<String>,
  sources: Vec<String>,
}

impl Subject {

  fn person_birth(name: &str, gender: &str, rodden_value: i32) -> Subject {
    Subject{
      name: name.to_string(),
      r#type: "person".to_string(),
      notes: "".to_string(),
      gender: gender.to_string(),
      event_type: "birth".to_string(),
      rodden_value,
      alt_names: vec![],
      sources: vec![]
    }
  }

  fn person_event(name: &str, gender: &str, event_type: &str, rodden_value: i32) -> Subject {
    Subject{
      name: name.to_string(),
      r#type: "person".to_string(),
      notes: "".to_string(),
      gender: gender.to_string(),
      event_type: event_type.to_string(),
      rodden_value,
      alt_names: vec![],
      sources: vec![]
    }
  }

}

#[derive(Serialize, Deserialize, Debug)]
pub struct ITime {
  year: i32,
  #[serde(rename="dayNum")]
  day_num: u32,
  progress: f64,
  #[serde(rename="dayLength")]
  day_length: f64,
  #[serde(rename="isDayTime")]
  is_day_time: bool,
  #[serde(rename="dayBefore")]
  day_before: bool,
  muhurta: u8,
  ghati: u8,
  vighati: u8,
  lipta: f64,
  #[serde(rename="weekDayNum")]
  week_day_num: u8,
}

impl ITime {

  pub fn new(ref_jd: f64, prev_rise_jd: f64, rise_jd: f64, set_jd: f64, next_rise_jd: f64, offset_secs: i16) -> ITime {
    let day_before = ref_jd < rise_jd;
    let is_day_time = !day_before && ref_jd < set_jd;
    let day_start = if day_before { prev_rise_jd } else { rise_jd };
    let day_length = if day_before { rise_jd - prev_rise_jd } else { next_rise_jd - rise_jd };
    let offset_jd = offset_secs as f64 / 86400f64;
    let dt = julian_day_to_datetime(ref_jd + offset_jd);
    let year = dt.year();
    let day_num = dt.ordinal() as u32;
    let progress = (ref_jd - day_start) / day_length;
    let muhurta = (progress * 30f64).floor() as u8;
    let ghati = (progress * 60f64).floor() as u8;
    let vighati = ((progress * 1800f64).floor() % 60f64) as u8;
    let lipta = (progress * 1800f64).floor() % 60f64;
    let iso_week_day_num = dt.weekday() as u8 + 1;
    let week_day_num = if iso_week_day_num == 7  { 1 } else { iso_week_day_num + 1 };
    ITime {
      year,
      day_num,
      progress,
      day_length,
      is_day_time,
      day_before,
      muhurta,
      ghati,
      vighati,
      lipta,
      week_day_num
    }
  }

}

#[derive(Serialize, Deserialize, Debug)]
pub enum MixedValue {
  StringVal(String),
  NumVal(f64),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ObjectMatch {
  key: String,
  r#type: String,
  value: MixedValue,
  #[serde(rename="revVal")]
  ref_val: f64,
}

impl ObjectMatch {
  fn new_f64(key: &str, r#type: &str, val: f64) -> ObjectMatch {
    ObjectMatch{ 
      key: key.to_string(),
      r#type: r#type.to_string(),
      value: MixedValue::NumVal(val),
      ref_val: 0f64
    }
  }
  fn new_string(key: &str, r#type: &str, val: &str, ref_val: f64) -> ObjectMatch {
    ObjectMatch{ 
      key: key.to_string(),
      r#type: r#type.to_string(),
      value: MixedValue::StringVal(val.to_string()),
      ref_val: ref_val
    }
  }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ProgressItem {
  pd: f64,
  jd: f64,
  bodies: Vec<KeyNumValue>,
  ayanamsha: f64
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ObjectMatchSet {
  num: u8,
  items: Vec<ObjectMatch>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AstroChart {
  _id: Option<String>,
  user: Option<String>,
  #[serde(rename="isDefaultBirthChart")]
  is_default_birth_chart: bool,
  subject: Subject,
  status: String,
  parent: Option<String>,
  jd: f64,
  datetime: NaiveDateTime,
  geo: GeoPos,
  placenames: Vec<Placename>,
  tz: String,
  #[serde(rename="tzOffset")]
  tz_offset: f64,
  ascendant: f64,
  mc: f64,
  vertex: f64,
  grahas: Vec<Graha>,
  houses: Vec<HouseSystem>,
  #[serde(rename="indianTime")]
  indian_time: ITime,
  ayanamshas: Vec<KeyNumValue>,
  upagrahas: Vec<KeyNumValue>,
  sphutas: Vec<VariantSet>,
  #[serde(rename="numValues")]
  num_values: Vec<KeyNumValue>,
  #[serde(rename="stringValues")]
  string_values: Vec<KeyStringValue>,
  objects: Vec<ObjectMatchSet>,
  rashis: Vec<RashiSet>,
  #[serde(rename="progressItems")]
  progress_items: Vec<ProgressItem>,
  notes: String,
  #[serde(rename="createdAt")]
  created_at: NaiveDateTime,
  #[serde(rename="modifiedAt")]
  modified_at: NaiveDateTime
}

impl AstroChart {

  pub fn get_ayanamsha_value(&self, key: &str) -> f64 {
    if let Some(row) = &self.ayanamshas.iter().find(|row| row.key == key.to_string()) {
      row.value
    } else {
      0f64
    }
  }

  pub fn adjusted_grahas(self, aya_key: &str) -> Vec<Graha> {
    let aya_val = self.get_ayanamsha_value(aya_key);
    self.grahas.iter().map(|g| Graha::adjusted(g.clone(), aya_val)).collect::<Vec<Graha>>()
  }

  pub fn descendant(&self) -> f64 {
    (&self.ascendant + 180f64) % 360f64
  }

  pub fn graha(&self, key: &str) -> Graha {
    if let Some(gr) = self.grahas.iter().find(|gr| gr.key == key.to_string()) { 
      gr.clone()
    } else {
      match key {
        "as" => Graha::simple("as", self.ascendant),
        "ds" => Graha::simple("ds", self.descendant()),
        _ => Graha::simple("-", 0f64)
      }
    }
  }

}