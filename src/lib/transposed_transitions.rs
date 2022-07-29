use serde::{Serialize, Deserialize};
use super::julian_date::*;
use super::models::{geo_pos::*, graha_pos::*};
use super::{models::{general::{KeyNumValue, KeyNumValueSet}}};
use super::{core::{calc_altitude, calc_body_jd_geo, calc_body_jd_topo}, transitions::{TransitionSet, AltTransitionSet, get_pheno_result}};

const MINS_PER_DAY: i32 = 1440;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AltitudeSample {
  pub mode: String,
  pub mins: f64,
  pub jd: f64,
  pub value: f64,
}

impl AltitudeSample {
  
  pub fn new(mode: &str, mins: f64, jd: f64, value: f64) -> Self {
    AltitudeSample{
      mode: mode.to_string(),
      mins: mins,
      jd: jd,
      value: value,
    }
  }

  pub fn basic_var(mode: &str, value: f64) -> Self {
    AltitudeSample{
      mode: mode.to_string(),
      mins: 0f64,
      jd: 0f64,
      value
    }
  }
  pub fn basic(mode: &str) -> Self {
    AltitudeSample::basic_var(mode, 0f64)
  }

  pub fn basic_low(mode: &str) -> Self {
    AltitudeSample::basic_var(mode, -90f64)
  }

  pub fn basic_high(mode: &str) -> Self {
    AltitudeSample::basic_var(mode, 90f64)
  }

  pub fn datetime_string(&self) -> String {
    return if self.jd > 0f64 { julian_day_to_iso_datetime(self.jd) } else { "".to_string() };
  }

  pub fn set_mode(&mut self, mode: &str)  {
    self.mode = mode.to_string();
  }

  pub fn to_key_num(&self) -> KeyNumValue {
    KeyNumValue::new(self.mode.as_str(), self.jd)
  }

}

pub fn calc_mid_point(first: AltitudeSample, second: AltitudeSample) -> f64 {
  let value_diff = second.value - first.value;
  let progress = second.value / value_diff;
  let jd_diff = second.jd - first.jd;
  second.jd - (jd_diff * progress)
}


fn calc_mid_sample(
  item: AltitudeSample,
  prev_min: f64,
  prev_value: f64,
  prev_jd: f64,
  mode: &str,
) -> AltitudeSample {
  let prev_sample = AltitudeSample::new(mode, prev_min, prev_jd, prev_value );
  let mid_point = calc_mid_point(prev_sample, item.clone());
  AltitudeSample::new(mode, prev_min, mid_point, 0f64)
}

fn recalc_min_max_transit_sample(
  sample: AltitudeSample,
  geo: GeoPos,
  lng: f64,
  lat: f64,
  max_mode: bool,
  multiplier: u8,
) -> AltitudeSample {
  let sample_rate = 0.25f64;
  let mut new_sample = sample;
  let num_sub_samples = multiplier as f64 * 2 as f64 * (1f64 / sample_rate);
  let sample_start_jd = new_sample.jd - num_sub_samples / (2f64 / sample_rate) / MINS_PER_DAY as f64;
  let sample_start_min = new_sample.mins - num_sub_samples / (2f64 / sample_rate);
  let mode = match max_mode { 
    true => "mc",
    false => "ic",
    _ => ""
   };
  let max = num_sub_samples as i32 + 1;
  for i in 0..max {
    let mins = sample_start_min + i as f64 * sample_rate;
    let jd = sample_start_jd + (i as f64 * sample_rate) / MINS_PER_DAY as f64;
    
    let value = calc_altitude(jd, false, geo.lat, geo.lng, lng, lat);
    let item = AltitudeSample::new(mode, mins, jd, value);
    if max_mode && item.value > new_sample.value {
      new_sample = item;
    } else if !max_mode && item.value < new_sample.value {
      new_sample = item;
    }
  }
  new_sample
}

#[derive(Debug, Copy, Clone)]
pub enum TransitionFilter {
  Rise = 1,
  Set = 2,
  Ic = 3,
  Mc = 4,
  RiseSet = 5,
  McIc = 6,
  All = 7,
}

impl TransitionFilter {
  pub fn match_rise(self) -> bool {
      match &self {
          TransitionFilter::Rise | TransitionFilter::RiseSet | TransitionFilter::All => true,
          _ => false, 
      }
  }

  pub fn match_set(self) -> bool {
      match &self {
          TransitionFilter::Set | TransitionFilter::RiseSet | TransitionFilter::All => true,
          _ => false, 
      }
  }

  pub fn match_mc(self) -> bool {
      match &self {
          TransitionFilter::Mc | TransitionFilter::McIc | TransitionFilter::All => true,
          _ => false, 
      }
  }

  pub fn match_ic(self) -> bool {
      match &self {
          TransitionFilter::Ic | TransitionFilter::McIc | TransitionFilter::All => true,
          _ => false, 
      }
  }
}

pub fn calc_transposed_object_transitions (
  jd_start: f64,
  geo: GeoPos,
  lng: f64,
  lat: f64,
  lng_speed: f64,
  multiplier: u8,
  filter: TransitionFilter,
  sample_key: &str,
  rise_set_minmax: bool
) -> Vec<AltitudeSample> {
  let max = MINS_PER_DAY / multiplier as i32 + 1;
  let mut items: Vec<AltitudeSample> = Vec::new();
  let match_set = filter.match_set();
  let match_rise = filter.match_rise();
  let match_mc = filter.match_mc();
  let match_ic = filter.match_ic();
  let mut ic = AltitudeSample::basic_high("ic");
  let mut rise = AltitudeSample::basic("rise");
  let mut set = AltitudeSample::basic("set");
  let mut mc = AltitudeSample::basic_low("mc");
  let mut prev_value = 0f64;
  let mut prev_min = 0f64;
  let mut prev_jd = 0f64;
  // resample the longitude and latitude speed for the moon only
  let mut disc_offset = 0f64;
  if sample_key == "su" || sample_key == "mo" {
    let pheno = get_pheno_result(jd_start, sample_key, 0i32);
    disc_offset = pheno.apparent_diameter_of_disc / 4f64;
  }
  let resample_speed = sample_key == "mo" && lng_speed != 0f64;
  for i in 0..max {
    let n = i as f64 * multiplier as f64;
    let day_frac = n / MINS_PER_DAY as f64;
    let jd = jd_start + day_frac;
    let mut sample_spd = lng_speed;
    let mut lat_spd = 0f64;
    if resample_speed {
      let sample_body = calc_body_jd_topo(jd, sample_key, geo, 0f64);
      sample_spd = sample_body.lng_speed;
      lat_spd = sample_body.lat_speed;
    }
    let adjusted_lng = if lng_speed != 0f64  { lng + sample_spd * day_frac } else { lng };
    let adjusted_lat = if lat_spd != 0f64 { lat + lat_spd * day_frac } else { lat };
    let value = calc_altitude(jd, false, geo.lat, geo.lng, adjusted_lng, adjusted_lat);

    let mut item = AltitudeSample::new("", n,jd, value);
    if match_mc && value > mc.value {
      item.set_mode("mc");
      mc = item.clone();
    }
    if match_ic && value < ic.value {
      item.set_mode("ic");
      ic = item.clone();
    }
    let offset_pv = prev_value + disc_offset;
    let offset_v = value + disc_offset;
    let offset_pv2 = prev_value - disc_offset;
    let offset_v2 = value - disc_offset;
    if match_rise && offset_pv < 0f64 && offset_v > 0f64 {
      rise = calc_mid_sample(item.clone(), prev_min, offset_pv, prev_jd, "rise");
    } else if match_set && offset_pv2 > 0f64 && offset_v2 < 0f64 {
      set = calc_mid_sample(item.clone(), prev_min, offset_pv2, prev_jd, "set");
    }
    if !match_mc && !match_ic {
      if !match_rise && match_set && set.jd > 0f64 {
        break;
      } else if !match_set && match_rise && rise.jd > 0f64 {
        break;
      }
    }
    items.push(item);
    prev_value = value;
    prev_min = n;
    prev_jd = jd;
  }
  if match_mc && mc.jd > 0f64 {
    mc = recalc_min_max_transit_sample(mc, geo.clone(), lng, lat, true, multiplier);
  }
  if match_ic && ic.jd > 0f64 {
    ic = recalc_min_max_transit_sample(ic, geo.clone(), lng, lat, false, multiplier);
  }
  if rise_set_minmax {
    if rise.jd <= 0f64 { 
      let rise_jd = if mc.value > 0f64 { 0f64 } else { mc.value };
      rise = AltitudeSample::new("rise", 0f64, rise_jd, mc.value - ic.value);
    }
    if set.jd <= 0f64 { 
      let set_jd = if mc.value > 0f64 { ic.value } else { 0f64 };
      set = AltitudeSample::new("set", 0f64, set_jd, mc.value - ic.value);
    }
  }
  vec![rise, set, mc, ic]
}

pub fn calc_transposed_graha_transition(
  jd_start: f64,
  geo: GeoPos,
  graha_pos: GrahaPos,
  filter: TransitionFilter,
  multiplier: u8,
) -> Vec<AltitudeSample> {
  calc_transposed_object_transitions(
    jd_start,
    geo,
    graha_pos.lng,
    graha_pos.lat,
    graha_pos.lng_speed,
    multiplier,
    filter,
    graha_pos.key.as_str(),
    true
  )
}

/*
  Calculate transposed transitions from a set of pre-calculated real celestial body positions
  This is useful when working with existing chart data for things like natal transitions
*/
pub fn calc_transposed_graha_transitions_from_source_positions(jd_start: f64, geo: GeoPos, graha_positions: Vec<GrahaPos>) -> Vec<KeyNumValueSet> {
  let mut key_num_sets: Vec<KeyNumValueSet> = Vec::new();
  for graha_pos in graha_positions {
    let tr_samples: Vec<AltitudeSample> = calc_transposed_object_transitions(
      jd_start,
      geo,
      graha_pos.lng,
      graha_pos.lat,
      graha_pos.lng_speed,
      5,
      TransitionFilter::All,
      graha_pos.key.as_str(),
      true
    );
    let tr_key_set: KeyNumValueSet = KeyNumValueSet::new(graha_pos.key.as_str(), tr_samples.iter().map(|tr| tr.to_key_num()).collect());
    key_num_sets.push(tr_key_set);
  }
  key_num_sets
}

/*
  Calculate transposed transitions from a set of real body positions with a different time and place
*/
pub fn calc_transposed_graha_transitions_from_source_refs(mode: &str, jd_start: f64, geo: GeoPos, jd_historic: f64, geo_historic: GeoPos, keys: Vec<String>, days: u16) -> Vec<KeyNumValueSet> {
  let mut key_num_sets: Vec<KeyNumValueSet> = Vec::new();
  for key in keys {
    let graha_pos = match mode {
      "topo" => calc_body_jd_topo(jd_historic, key.as_str(), geo_historic, 0f64),
      _ => calc_body_jd_geo(jd_historic, key.as_str(), 0f64)
    };
    let mut items: Vec<KeyNumValue> = Vec::new();
    for i in 0..days {
      let ref_jd = jd_start + i as f64;
      let tr_samples: Vec<AltitudeSample> = calc_transposed_object_transitions(
        ref_jd,
        geo,
        graha_pos.lng,
        graha_pos.lat,
        graha_pos.lng_speed,
        5,
        TransitionFilter::All,
        graha_pos.key.as_str(),
        true
      );
      let mut new_items: Vec<KeyNumValue> = tr_samples.iter().map(|tr| tr.to_key_num()).collect();
      items.append(&mut new_items);
    }
    let tr_key_set: KeyNumValueSet = KeyNumValueSet::new(graha_pos.key.as_str(), items);
    key_num_sets.push(tr_key_set);
  }
  key_num_sets
}

fn extract_from_alt_samples(alt_samples: &Vec<AltitudeSample>, key: &str) -> AltitudeSample {
  alt_samples.into_iter().find(|sample| sample.mode.as_str() == key).unwrap_or(&AltitudeSample::basic(key)).to_owned()
}


/**
 * Alternative method to fetch transitions for near polar latitudes (> +60 and < -60) based on altitudes
*/
pub fn calc_transitions_from_source_refs_altitude(jd: f64, key: &str, geo: GeoPos) -> TransitionSet {
  let pos = calc_body_jd_topo(jd, key, geo, 0f64);
  let alt_samples = calc_transposed_object_transitions(jd, geo, pos.lng, pos.lat, pos.lng_speed, 5, TransitionFilter::All, key, true);
  let rise = extract_from_alt_samples(&alt_samples, "rise");
  let set = extract_from_alt_samples(&alt_samples, "set");
  let mc = extract_from_alt_samples(&alt_samples, "mc");
  let ic = extract_from_alt_samples(&alt_samples, "ic");
  TransitionSet { 
    rise: rise.jd,
    mc: mc.jd,
    set: set.jd,
    ic: ic.jd,
  }
}

/**
 * Alternative method to fetch transitions for near polar latitudes (> +60 and < -60) with min and max altitudes
*/
pub fn calc_transitions_from_source_refs_minmax(jd: f64, key: &str, geo: GeoPos) -> AltTransitionSet {
  let pos = calc_body_jd_topo(jd, key, geo, 0f64);
  let alt_samples = calc_transposed_object_transitions(jd, geo, pos.lng, pos.lat, pos.lng_speed, 5, TransitionFilter::All, key, false);
  let rise = extract_from_alt_samples(&alt_samples, "rise");
  let set = extract_from_alt_samples(&alt_samples, "set");
  let mc = extract_from_alt_samples(&alt_samples, "mc");
  let ic = extract_from_alt_samples(&alt_samples, "ic");
  AltTransitionSet { 
    min: ic.value,
    rise: rise.jd,
    mc: mc.jd,
    set: set.jd,
    ic: ic.jd,
    max: mc.value,
  }
}

pub fn calc_transposed_graha_transitions_from_source_refs_topo(jd_start: f64, geo: GeoPos, jd_historic: f64, geo_historic: GeoPos, keys: Vec<String>, days: u16) -> Vec<KeyNumValueSet> {
  calc_transposed_graha_transitions_from_source_refs("topo", jd_start, geo, jd_historic, geo_historic, keys, days)
}

/*
  Calculate transposed transitions from a set of real body positions with a different time with geocentric positions
*/
pub fn calc_transposed_graha_transitions_from_source_refs_geo(jd_start: f64, geo: GeoPos, jd_historic: f64, geo_historic: GeoPos, keys: Vec<String>, days: u16) -> Vec<KeyNumValueSet> {
  calc_transposed_graha_transitions_from_source_refs("geo", jd_start, geo, jd_historic, geo_historic, keys, days)
}
