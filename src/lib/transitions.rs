use serde::{Serialize, Deserialize};
use super::super::extensions::swe::{rise_trans};
use libswe_sys::sweconst::{Bodies};
use libswe_sys::swerust::{handler_swe07::{pheno_ut}};
use super::{core::{calc_altitude_object, calc_next_prev_horizon}, traits::*, models::{geo_pos::*, general::*, graha_pos::{PhenoResult, PhenoItem}, chart::{ITime}}, transposed_transitions::{calc_transitions_from_source_refs_altitude, calc_transitions_from_source_refs_minmax}, julian_date::{julian_day_to_iso_datetime}};

pub enum TransitionParams {
  Rise = 1,
  Set = 2,
  Mc = 4,
  Ic = 8,
  Center = 256,
  Bottom = 8192,
  Fixed = 16384,
  BitNoRefraction = 512,
  BitGeoctrNoEclLat = 128
}

impl TransitionParams {
  pub fn center_disc_rising() -> i32 {
    TransitionParams::Center as i32 | TransitionParams::BitNoRefraction as i32 | TransitionParams::BitGeoctrNoEclLat as i32
  }

  pub fn bottom_disc_rising() -> i32 {
    TransitionParams::Bottom as i32 | TransitionParams::BitNoRefraction as i32 | TransitionParams::BitGeoctrNoEclLat as i32
  }

  pub fn normal() -> i32 {
    TransitionParams::BitNoRefraction as i32 | TransitionParams::BitGeoctrNoEclLat as i32
  }

  pub fn center_disc_rising_rise() -> i32 {
    TransitionParams::center_disc_rising() | TransitionParams::Rise as i32
  }

  pub fn rise_normal() -> i32 {
    TransitionParams::Fixed as i32 | TransitionParams::Rise as i32
  }

  pub fn set_normal() -> i32 {
    TransitionParams::Fixed as i32 | TransitionParams::Set as i32
  }

  pub fn center_disc_rising_set() -> i32 {
    TransitionParams::center_disc_rising() | TransitionParams::Set as i32
  }

  pub fn mc() -> i32 {
    TransitionParams::BitNoRefraction as i32 | TransitionParams::Mc as i32
  }

  pub fn ic() -> i32 {
    TransitionParams::BitNoRefraction as i32 | TransitionParams::Ic as i32
  }
}

pub trait TransitionGroup {
  fn period(&self) -> f64;

  fn to_key_nums(&self) -> Vec<KeyNumValue>;
}


#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ExtendedTransitionSet {
  #[serde(rename="nextRise")]
  pub prev_set: f64,
  pub rise: f64,
  pub mc: f64,
  pub set: f64,
  pub ic: f64,
  #[serde(rename="nextRise")]
  pub next_rise: f64,
  pub min: f64,
  pub max: f64,
}

impl TransitionGroup for ExtendedTransitionSet {
  fn period(&self) -> f64 {
    self.set - self.rise
  }

  fn to_key_nums(&self) -> Vec<KeyNumValue> {
    let is_up = self.min >= 0f64 && self.max > 0f64;
    let prev_key = if is_up { "prev_rise" } else { "prev_set"};
    let next_key = if is_up { "next_set" } else { "next_rise"};
    vec![
      KeyNumValue::new(prev_key, self.prev_set),
      KeyNumValue::new("rise", self.rise),
      KeyNumValue::new("mc", self.mc),
      KeyNumValue::new("set", self.set),
      KeyNumValue::new("ic", self.ic),
      KeyNumValue::new(next_key, self.next_rise),
      KeyNumValue::new("min", self.min),
      KeyNumValue::new("max", self.max),
    ]
  }
}

impl ExtendedTransitionSet {
  fn to_iso_datetimes(&self) -> Vec<FlexiValue> {
    let is_up = self.min >= 0f64 && self.max > 0f64;
    let prev_key = if is_up { "prev_rise" } else { "prev_set"};
    let next_key = if is_up { "next_set" } else { "next_rise"};
    vec![
      FlexiValue::NumValue(KeyNumValue::new("min", self.min)),
      FlexiValue::StringValue(KeyNumValue::new(prev_key, self.prev_set).as_iso_string()),
      FlexiValue::StringValue(KeyNumValue::new("rise", self.rise).as_iso_string()),
      FlexiValue::StringValue(KeyNumValue::new("mc", self.rise).as_iso_string()),
      FlexiValue::StringValue(KeyNumValue::new("set", self.rise).as_iso_string()),
      FlexiValue::StringValue(KeyNumValue::new("ic", self.rise).as_iso_string()),
      FlexiValue::StringValue(KeyNumValue::new(next_key, self.next_rise).as_iso_string()),
      FlexiValue::NumValue(KeyNumValue::new("max", self.max)),
    ]
  }

  pub fn is_up(&self) -> bool {
    (self.rise == 0f64 || self.set == 0f64) && self.min >= -0.5f64
  }

  pub fn is_down(&self) -> bool {
    (self.rise == 0f64 || self.set == 0f64) && self.max <= 0.5f64
  }

  pub fn start_mode(&self) -> i8 {
    if self.is_up() { 1 } else if self.is_down() { -1 } else { 0 }
  }

  pub fn as_iso_datetime(&self) -> ExtendedTransitionIsoSet {
    let prev_rise_val = if self.is_up() { self.prev_set } else { 0f64 };
    let prev_set_val = if self.is_up() { 0f64 } else { self.prev_set };
    let next_rise_val = if self.is_up() { 0f64 } else { self.next_rise };
    let next_set_val = if self.is_up() { self.next_rise } else { 0f64 };
    ExtendedTransitionIsoSet{
      min: self.min,
      prev_rise: julian_day_to_iso_datetime(prev_rise_val),
      prev_set: julian_day_to_iso_datetime(prev_set_val),
      rise: julian_day_to_iso_datetime(self.rise),
      mc: julian_day_to_iso_datetime(self.mc),
      set: julian_day_to_iso_datetime(self.set),
      ic: julian_day_to_iso_datetime(self.ic),
      next_rise: julian_day_to_iso_datetime(next_rise_val),
      next_set: julian_day_to_iso_datetime(next_set_val),
      max: self.max,
    }
  }

  pub fn to_value_set(&self, iso_mode: bool) -> AltTransitionValueSet {
    match iso_mode {
      true => AltTransitionValueSet::ExtendedIsoValues(self.as_iso_datetime()),
      _ => AltTransitionValueSet::ExtendedJdValues(self.to_owned())
    }
  }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ExtendedTransitionIsoSet {
  #[serde(rename="prevSet",skip_serializing_if = "String::is_empty")]
  pub prev_set: String,
  #[serde(rename="prevRise",skip_serializing_if = "String::is_empty")]
  pub prev_rise: String,
  #[serde(skip_serializing_if = "String::is_empty")]
  pub rise: String,
  #[serde(skip_serializing_if = "String::is_empty")]
  pub mc: String,
  #[serde(skip_serializing_if = "String::is_empty")]
  pub set: String,
  pub ic: String,
  #[serde(rename="nextRise",skip_serializing_if = "String::is_empty")]
  pub next_rise: String,
  #[serde(rename="nextSet",skip_serializing_if = "String::is_empty")]
  pub next_set: String,
  pub min: f64,
  pub max: f64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AltTransitionSet {
  pub min: f64,
  pub rise: f64,
  pub mc: f64,
  pub set: f64,
  pub ic: f64,
  pub max: f64,
}

impl AltTransitionSet {
 /*  fn to_iso_datetimes(&self) -> Vec<FlexiValue> {
    vec![
      FlexiValue::NumValue(KeyNumValue::new("min", self.min)),
      FlexiValue::StringValue(KeyNumValue::new("rise", self.rise).as_iso_string()),
      FlexiValue::StringValue(KeyNumValue::new("mc", self.rise).as_iso_string()),
      FlexiValue::StringValue(KeyNumValue::new("set", self.rise).as_iso_string()),
      FlexiValue::StringValue(KeyNumValue::new("ic", self.rise).as_iso_string()),
      FlexiValue::NumValue(KeyNumValue::new("max", self.max)),
    ]
  } */
  
  pub fn is_up(&self) -> bool {
    (self.rise == 0f64 || self.set == 0f64) && self.min >= -0.5f64
  }

  pub fn is_down(&self) -> bool {
    (self.rise == 0f64 || self.set == 0f64) && self.max <= 0.5f64
  }

  pub fn start_mode(&self) -> i8 {
    if self.is_up() { 1 } else if self.is_down() { -1 } else { 0 }
  }

  pub fn as_iso_datetime(&self) -> AltTransitionIsoSet {
    AltTransitionIsoSet{
      min: self.min,
      rise: julian_day_to_iso_datetime(self.rise),
      mc: julian_day_to_iso_datetime(self.mc),
      set: julian_day_to_iso_datetime(self.set),
      ic: julian_day_to_iso_datetime(self.ic),
      max: self.max,
    }
  }

  pub fn to_value_set(&self, iso_mode: bool) -> AltTransitionValueSet {
    match iso_mode {
      true => AltTransitionValueSet::IsoValues(self.as_iso_datetime()),
      _=> AltTransitionValueSet::JdValues(self.to_owned()),
    }
  }

}

impl TransitionGroup for AltTransitionSet {
  fn period(&self) -> f64 {
    self.set - self.rise
  }

  fn to_key_nums(&self) -> Vec<KeyNumValue> {
    vec![
      KeyNumValue::new("min", self.min),
      KeyNumValue::new("rise", self.rise),
      KeyNumValue::new("mc", self.mc),
      KeyNumValue::new("set", self.set),
      KeyNumValue::new("ic", self.ic),
      KeyNumValue::new("max", self.max),
    ]
  }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum AltTransitionValueSet {
  JdValues(AltTransitionSet),
  IsoValues(AltTransitionIsoSet),
  ExtendedJdValues(ExtendedTransitionSet),
  ExtendedIsoValues(ExtendedTransitionIsoSet),
}
/* 
 This serves only show rise, set, mc and ic times as ISO UTC strings.
*/
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AltTransitionIsoSet {
  pub min: f64,
  pub rise: String,
  pub mc: String,
  pub set: String,
  pub ic: String,
  pub max: f64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TransitionSet {
  pub rise: f64,
  pub mc: f64,
  pub set: f64,
  pub ic: f64,
}

impl TransitionGroup for TransitionSet {
  fn period(&self) -> f64 {
    self.set - self.rise
  }

  fn to_key_nums(&self) -> Vec<KeyNumValue> {
    let rise_key = if self.rise < 100f64 { "max" } else { "rise" };
    let set_key = if self.set < 100f64 { "min" } else { "set" };
    vec![
      KeyNumValue::new(rise_key, self.rise),
      KeyNumValue::new("mc", self.mc),
      KeyNumValue::new(set_key, self.set),
      KeyNumValue::new("ic", self.ic),
    ]
  }
}

pub fn is_near_poles(lat: f64) -> bool {
  lat >= 60f64 || lat <= -60f64
}

pub fn calc_transition_set_extended_fast(jd: f64, ipl: Bodies, lat: f64, lng: f64) -> ExtendedTransitionSet {
  let ref_jd = start_jd_geo(jd, lng);
  let prev_set = next_set(ref_jd - 1f64, ipl, lat, lng);
  let rise = next_rise(ref_jd, ipl, lat, lng);
  let set = next_set(ref_jd, ipl, lat, lng);

  //let mc = next_mc_q(ref_jd, ipl, lat, lng, rise);
  let mc = next_mc_normal(ref_jd, ipl, lat, lng);
  let ic = next_ic_normal(ref_jd, ipl, lat, lng);
  let next_rise = next_rise(set, ipl, lat, lng);
  let min = calc_altitude_object(ic, false, lat, lng, ipl.to_key());
  let max = calc_altitude_object(mc, false, lat, lng, ipl.to_key());
  ExtendedTransitionSet { 
    prev_set,
    rise,
    mc,
    set,
    ic,
    next_rise,
    min,
    max,
  }
}

pub fn calc_transition_set_alt_fast(jd: f64, ipl: Bodies, lat: f64, lng: f64) -> AltTransitionSet {
  let ref_jd = start_jd_geo(jd, lng);
  let rise = next_rise(ref_jd, ipl, lat, lng);
  let set = next_set(ref_jd, ipl, lat, lng);
  let mc = next_mc_normal(ref_jd, ipl, lat, lng);
  let ic = next_ic_normal(ref_jd, ipl, lat, lng);
  let min = calc_altitude_object(ic, false, lat, lng, ipl.to_key());
  let max = calc_altitude_object(mc, false, lat, lng, ipl.to_key());
  AltTransitionSet { 
    min,
    rise,
    mc,
    set,
    ic,
    max
  }
}

pub fn calc_transition_set_extended_azalt(jd: f64, ipl: Bodies, lat: f64, lng: f64) -> ExtendedTransitionSet {
  let ref_jd = start_jd_geo(jd, lng);
  let geo = GeoPos::simple(lat, lng);
  let ref_key = ipl.to_key();
  let base = calc_transitions_from_source_refs_minmax(ref_jd, ref_key, geo);
  /* let prev = calc_transitions_from_source_refs_altitude(ref_jd - 1f64, ref_key, geo);
  let next = calc_transitions_from_source_refs_altitude(ref_jd + 1f64, ref_key, geo); */
  let prev = calc_transitions_from_source_refs_altitude(ref_jd - 1f64, ref_key, geo);
  let next = calc_transitions_from_source_refs_altitude(ref_jd + 1f64, ref_key, geo);
  let mut prev_set = prev.set;
  let mut next_rise = next.rise;
  if prev.rise < 100f64 || prev.set < 100f64 {
    let down = base.min < 0f64 && base.max < 0f64;
    prev_set = calc_next_prev_horizon(jd, lat, lng, ipl.to_key(), down, false);
    next_rise = calc_next_prev_horizon(jd, lat, lng, ipl.to_key(), down, true);
  }
  ExtendedTransitionSet { 
    prev_set,
    rise: base.rise,
    mc: base.mc,
    set: base.set,
    ic: base.ic,
    next_rise,
    min: base.min,
    max: base.max,
  }
}

pub fn calc_transition_set_alt_azalt(jd: f64, ipl: Bodies, lat: f64, lng: f64) -> AltTransitionSet {
  let ref_jd = start_jd_geo(jd, lng);
  let geo = GeoPos::simple(lat, lng);
  let ref_key = ipl.to_key();
  calc_transitions_from_source_refs_minmax(ref_jd, ref_key, geo)
}


pub fn calc_transition_set_extended(jd: f64, ipl: Bodies, lat: f64, lng: f64) -> ExtendedTransitionSet {
  if is_near_poles(lat) {
    calc_transition_set_extended_azalt(jd, ipl, lat, lng)
  } else {
    calc_transition_set_extended_fast(jd, ipl, lat, lng)
  }
}

pub fn calc_transition_set_alt(jd: f64, ipl: Bodies, lat: f64, lng: f64) -> AltTransitionSet {
  if is_near_poles(lat) {
    calc_transition_set_alt_azalt(jd, ipl, lat, lng)
  } else {
    calc_transition_set_alt_fast(jd, ipl, lat, lng)
  }
}

pub fn calc_transition_set_fast(jd: f64, ipl: Bodies, lat: f64, lng: f64) -> TransitionSet {
  let ref_jd = start_jd_geo(jd, lng);
  let rise = next_rise(ref_jd, ipl, lat, lng);
  let set = next_set(rise, ipl, lat, lng);
  /* let mc = next_mc(ref_jd, ipl, lat, lng);
  let ic = next_ic(ref_jd, ipl, lat, lng); */
  // MC/IC flags have issues via alc_mer_trans when compiled with gcc
  // use median of rise/set with fixed disc instead
  let mc = next_mc_normal(ref_jd, ipl, lat, lng);
  let ic = next_ic_normal(mc, ipl, lat, lng);
  TransitionSet { 
    rise,
    mc,
    set,
    ic,
  }
}

pub fn calc_transition_set(jd: f64, ipl: Bodies, lat: f64, lng: f64) -> TransitionSet {
  if is_near_poles(lat) {
    let ref_jd = start_jd_geo(jd, lng);
    calc_transitions_from_source_refs_altitude(ref_jd, ipl.to_key(), GeoPos::simple(lat, lng))
  } else {
    calc_transition_set_fast(jd, ipl, lat, lng)
  }
}

pub fn calc_transition_sun(jd: f64, geo: GeoPos) -> ExtendedTransitionSet {
  calc_transition_set_extended(jd, Bodies::Sun, geo.lat, geo.lng)
}

pub fn calc_transitions_sun(jd: f64, days: u16, geo: GeoPos) -> Vec<KeyNumValue> {
  let mut sets: Vec<KeyNumValue> = Vec::new();
  for i in 0..days {
    let ref_jd = jd + i as f64;
    let items = calc_transition_set_alt(ref_jd, Bodies::Sun, geo.lat, geo.lng).to_key_nums();
    for item in items {
      sets.push(item);
    }
  }
  sets
}

pub fn calc_transition_moon(jd: f64, geo: GeoPos) -> ExtendedTransitionSet {
  calc_transition_set_extended(jd, Bodies::Moon, geo.lat, geo.lng)
}

pub fn calc_transition_body(jd: f64, ipl: Bodies, geo: GeoPos) -> TransitionSet {
  calc_transition_set(jd, ipl, geo.lat, geo.lng)
}

pub fn next_rise(tjd_ut: f64, ipl: Bodies, lat: f64, lng: f64) -> f64 {
  rise_trans(tjd_ut, ipl, lat, lng, TransitionParams::center_disc_rising_rise())
}

pub fn next_set(tjd_ut: f64, ipl: Bodies, lat: f64, lng: f64) -> f64 {
  rise_trans(tjd_ut, ipl, lat, lng, TransitionParams::center_disc_rising_set())
}

pub fn next_rise_normal(tjd_ut: f64, ipl: Bodies, lat: f64, lng: f64) -> f64 {
  rise_trans(tjd_ut, ipl, lat, lng, TransitionParams::rise_normal())
}

pub fn next_set_normal(tjd_ut: f64, ipl: Bodies, lat: f64, lng: f64) -> f64 {
  rise_trans(tjd_ut, ipl, lat, lng, TransitionParams::set_normal())
}

pub fn next_mc_normal(tjd_ut: f64, ipl: Bodies, lat: f64, lng: f64) -> f64 {
  let rise_n = rise_trans(tjd_ut, ipl, lat, lng, TransitionParams::rise_normal());
  let set_n = rise_trans(rise_n, ipl, lat, lng, TransitionParams::set_normal());
  (set_n + rise_n) / 2f64
}

pub fn next_ic_normal(tjd_ut: f64, ipl: Bodies, lat: f64, lng: f64) -> f64 {
  let set_n = rise_trans(tjd_ut, ipl, lat, lng, TransitionParams::set_normal());
  let next_rise_n = rise_trans(set_n, ipl, lat, lng, TransitionParams::rise_normal());
  (next_rise_n + set_n) / 2f64
}

pub fn next_mc(tjd_ut: f64, ipl: Bodies, lat: f64, lng: f64) -> f64 {
  rise_trans(tjd_ut, ipl, lat, lng, TransitionParams::mc())
}

pub fn next_mc_q(tjd_ut: f64, ipl: Bodies, lat: f64, lng: f64, rise_jd: f64) -> f64 {
let set_jd = rise_trans(tjd_ut, ipl, lat, lng, TransitionParams::Set as i32);
((set_jd - rise_jd) / 2f64) + rise_jd
}

pub fn next_ic(tjd_ut: f64, ipl: Bodies, lat: f64, lng: f64) -> f64 {
  rise_trans(tjd_ut, ipl, lat, lng, TransitionParams::ic())
}

pub fn next_ic_q(tjd_ut: f64, ipl: Bodies, lat: f64, lng: f64, set_jd: f64) -> f64 {
  let next_rise_jd = rise_trans(tjd_ut + 1f64, ipl, lat, lng, TransitionParams::Rise as i32);
  ((next_rise_jd - set_jd) / 2f64) + set_jd
}

pub fn start_jd_geo(jd: f64, lng: f64) -> f64 {
  let offset = (0f64 - lng / 15f64) / 24f64;
  let jd_progress = jd % 1f64;
  let adjusted_progress = offset - jd_progress;
  
  let start_offset = if adjusted_progress >= 0.5 { 0.5f64 } else { -0.5f64 };
  let start = jd.floor() + start_offset;
  let ref_jd = start + offset;
  let diff = jd - ref_jd;
  if diff > 1f64 {
    ref_jd + 1f64
  } else if diff < -1f64 {
    ref_jd - 1f64
  } else {
    ref_jd
  }
}

pub fn get_transition_sets(jd: f64, keys: Vec<&str>, geo: GeoPos) -> Vec<KeyNumValueSet> {
  let mut transit_sets: Vec<KeyNumValueSet> = Vec::new();
  for key in keys {
    let tr_set: Vec<KeyNumValue> = match key {
      "su" | "mo" => calc_transition_set_extended(jd, Bodies::from_key(key), geo.lat, geo.lng).to_key_nums(),
      _ => calc_transition_set(jd, Bodies::from_key(key), geo.lat, geo.lng).to_key_nums(),
    };
    transit_sets.push(KeyNumValueSet::new(key, tr_set));
  }
  transit_sets
}

pub fn get_transition_sets_extended(jd: f64, keys: Vec<String>, geo: GeoPos, days: u16) -> Vec<KeyNumValueSet> {
  let mut transit_sets: Vec<KeyNumValueSet> = Vec::new();
  for key in keys {
    let mut tr_set: Vec<KeyNumValue> = Vec::new();
    for i in 0..days {
      let ref_jd = jd + i as f64;
      let mut tr_set_day = calc_transition_set_alt(ref_jd, Bodies::from_key(key.as_str()), geo.lat, geo.lng).to_key_nums();
      tr_set.append(&mut tr_set_day);
    }
    transit_sets.push(KeyNumValueSet::new(key.as_str(), tr_set));
  }
  transit_sets
}


pub fn get_pheno_result(jd: f64, key: &str, iflag: i32) -> PhenoResult {
  let ipl = Bodies::from_key(key);
  let result = pheno_ut(jd, ipl, iflag);
  PhenoResult::new_from_result(result)
}

pub fn get_pheno_results(jd: f64, keys: Vec<&str>) -> Vec<PhenoItem> {
  let mut items: Vec<PhenoItem> = Vec::new();
  for key in keys {
    let ipl = Bodies::from_key(key);
    let result = pheno_ut(jd, ipl, 0i32);
    let item = PhenoItem::new_from_result(key, result);
    items.push(item);
  }
  items
}

pub fn to_indian_time_with_transitions(jd: f64, geo: GeoPos, offset_tz_secs: Option<i16>, iso_mode: bool) -> (ITime, AltTransitionValueSet, AltTransitionValueSet, AltTransitionValueSet, i16) {
  let current = calc_transition_set_extended(jd, Bodies::from_key("su"), geo.lat, geo.lng);
  let prev = calc_transition_set_alt(jd - 1f64, Bodies::from_key("su"), geo.lat, geo.lng);
  let next = calc_transition_set_alt(jd + 1f64, Bodies::from_key("su"), geo.lat, geo.lng);
  let prev_start = match prev.start_mode() {
    -1 => prev.mc,
    1 => prev.ic,
    _ => prev.rise,
  };
  let base_start = match current.start_mode() {
    -1 => current.mc,
    1 => if current.ic < jd { current.ic} else { prev.ic },
    _ => current.rise,
  };

  let base_set = match current.start_mode() {
    -1 => prev.mc,
    1 => next.ic,
    _ => current.set,
  };
  let next_start = match next.start_mode() {
    -1 => next.mc,
    1 => if current.ic < jd { next.ic} else { current.ic },
    _ => next.rise,
  };
  let offset_secs = if offset_tz_secs != None { offset_tz_secs.unwrap() } else { (geo.lng * 240f64) as i16 };
  (ITime::new(jd, prev_start, base_start, base_set, next_start, current.start_mode(), offset_secs), prev.to_value_set(iso_mode), current.to_value_set(iso_mode), next.to_value_set(iso_mode), offset_secs)
}

pub fn to_indian_time(jd: f64, geo: GeoPos, offset_tz_secs: Option<i16>) -> ITime {
  let (i_time, _, _, _, _) = to_indian_time_with_transitions(jd, geo, offset_tz_secs, false);
  i_time
}
