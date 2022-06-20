use serde::{Serialize, Deserialize};
use super::super::extensions::swe::{rise_trans};
use libswe_sys::sweconst::{Bodies};
use libswe_sys::swerust::{handler_swe07::{pheno_ut, PhenoUtResult}};
use super::{traits::*, models::{geo_pos::*, general::*}};

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
  pub prev_set: f64,
  pub rise: f64,
  pub mc: f64,
  pub set: f64,
  pub ic: f64,
  pub next_rise: f64,
}

impl TransitionGroup for ExtendedTransitionSet {
  fn period(&self) -> f64 {
    self.set - self.rise
  }

  fn to_key_nums(&self) -> Vec<KeyNumValue> {
    vec![
      KeyNumValue::new("prev_set", self.prev_set),
      KeyNumValue::new("rise", self.rise),
      KeyNumValue::new("mc", self.mc),
      KeyNumValue::new("set", self.set),
      KeyNumValue::new("ic", self.ic),
      KeyNumValue::new("next_rise", self.next_rise),
    ]
  }
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
    vec![
      KeyNumValue::new("rise", self.rise),
      KeyNumValue::new("mc", self.mc),
      KeyNumValue::new("set", self.set),
      KeyNumValue::new("ic", self.ic),
    ]
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

pub fn calc_transition_set_extended(jd: f64, ipl: Bodies, lat: f64, lng: f64) -> ExtendedTransitionSet {
  let ref_jd = start_jd_geo(jd, lng);
  let prev_set = next_set(ref_jd - 1f64, ipl, lat, lng);
  let rise = next_rise(ref_jd, ipl, lat, lng);
  let set = next_set(ref_jd, ipl, lat, lng);

  //let mc = next_mc_q(ref_jd, ipl, lat, lng, rise);
  let mc = next_mc(ref_jd, ipl, lat, lng);
  //let mc = 0f64;
  //let ic = 0f64;
  //let ic = next_ic_q(ref_jd, ipl, lat, lng, set);
  let ic = next_ic(ref_jd, ipl, lat, lng);
  let next_rise = next_rise(set, ipl, lat, lng);
  ExtendedTransitionSet { 
    prev_set,
    rise,
    mc,
    set,
    ic,
    next_rise
  }
}

pub fn calc_transition_set(jd: f64, ipl: Bodies, lat: f64, lng: f64) -> TransitionSet {
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
    rise: rise,
    mc: mc,
    set: set,
    ic: ic,
  }
}

pub fn calc_transition_sun(jd: f64, geo: GeoPos) -> ExtendedTransitionSet {
  calc_transition_set_extended(jd, Bodies::Sun, geo.lat, geo.lng)
}

pub fn calc_transitions_sun(jd: f64, days: i32, geo: GeoPos) -> Vec<KeyNumValue> {
  let mut sets: Vec<KeyNumValue> = Vec::new();
  for i in 0..days {
    let ref_jd = jd + i as f64;
    let items = calc_transition_set(ref_jd, Bodies::Sun, geo.lat, geo.lng).to_key_nums();
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
  let jd_progress = jd % 0f64;
  let adjusted_progress = jd_progress + offset;
  let start_offset = if adjusted_progress >= 0.5 { 0.5f64 } else { -0.5f64 };
  let start = jd.floor() + start_offset;
  start + offset
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


pub fn get_pheno_result(jd: f64, key: &str, iflag: i32) -> PhenoResult {
  let ipl = Bodies::from_key(key);
  let result = pheno_ut(jd, ipl, iflag);
  PhenoResult::new_from_result(result)
}
