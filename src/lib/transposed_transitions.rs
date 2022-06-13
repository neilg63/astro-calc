use serde::{Serialize, Deserialize};
use super::julian_date::*;
use super::models::{geo_pos::*, graha_pos::*};
use super::core::*;

const mins_day: i32 = 1440;

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

  pub fn basic(mode: &str) -> Self {
    AltitudeSample{
      mode: mode.to_string(),
      mins: 0f64,
      jd: 0f64,
      value: 0f64,
    }
  }

  pub fn iso_date(&self) -> String {
    return if self.jd > 0f64 { julian_day_to_iso_datetime(self.jd) } else { "".to_string() };
  }

  pub fn set_mode(&mut self, mode: &str)  {
    self.mode = mode.to_string();
  }

}

pub fn calc_mid_point(first: AltitudeSample, second: AltitudeSample) -> f64 {
  let valueDiff = second.value - first.value;
  let progress = second.value / valueDiff;
  let jdDiff = second.jd - first.jd;
  second.jd - jdDiff * progress
}


fn calc_mid_sample(
  item: AltitudeSample,
  prev_min: f64,
  prev_value: f64,
  prev_jd: f64,
  mode: &str,
) -> AltitudeSample {
  let prev_sample = AltitudeSample::new(mode, prev_min, prev_value, prev_jd );
  let mid_point = calc_mid_point(prev_sample, item);
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
  let sample_start_jd = new_sample.jd - num_sub_samples / (2f64 / sample_rate) / mins_day as f64;
  let sample_start_min = new_sample.mins - num_sub_samples / (2f64 / sample_rate);
  let mode = match max_mode { 
    true => "mc",
    false => "ic",
    _ => ""
   };
  let max = num_sub_samples as i32 + 1;
  for i in 0..max {
    let mins = sample_start_min + i as f64 * sample_rate;
    let jd = sample_start_jd + (i as f64 * sample_rate) / mins_day as f64;
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



enum Transits {
  Rise = 1,
  Set = 2,
  Mc = 4,
  Ic = 8,
  Center = 256,
  Bottom = 8192,
  BitNoRefraction = 512,
  BitGeoctrNoEclLat = 128
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
) -> Vec<AltitudeSample> {
  let max = mins_day / multiplier as i32;
  let mut items: Vec<AltitudeSample> = Vec::new();
  let match_set = filter.match_set();
  let match_rise = filter.match_rise();
  let match_mc = filter.match_mc();
  let match_ic = filter.match_ic();
  let mut ic = AltitudeSample::basic("ic");
  let mut rise = AltitudeSample::basic("rise");
  let mut set = AltitudeSample::basic("set");
  let mut mc = AltitudeSample::basic("mc");
  let mut prev_value = 0f64;
  let mut prev_min = 0f64;
  let mut prev_jd = 0f64;
  // resample the longitude and latitude speed for the moon only
  let resample_speed = sample_key == "mo" && lng_speed != 0f64;
  for i in 0..max {
    let n = i as f64 * multiplier as f64;
    let dayFrac = n / mins_day as f64;
    let jd = jd_start + dayFrac;
    let mut sample_spd = lng_speed;
    let mut latSpd = 0f64;
    if resample_speed {
      let sample_body = calc_body_jd(jd, sample_key, false, true);
      sample_spd = sample_body.lng_speed;
      latSpd = sample_body.lat_speed;
    }
    let adjusted_lng = if lng_speed != 0f64  { lng + sample_spd * dayFrac } else { lng };
    let adjusted_lat = if latSpd != 0f64 { lat + latSpd * dayFrac } else { lat };
    let value = calc_altitude(jd, false, geo.lat, geo.lng, adjusted_lng, adjusted_lat);
    let mut item = AltitudeSample::new("", n,jd, value);
    if match_mc && value > mc.value {
      item.set_mode("mc");
      mc = item.clone();
    } else if match_ic && value < ic.value {
      item.set_mode("ic");
      ic = item.clone();
    }
    if match_rise && prev_value < 0f64 && value > 0f64 {
      rise = calc_mid_sample(item.clone(), prev_min, prev_value, prev_jd, "rise");
    } else if match_set && prev_value > 0f64 && value < 0f64 {
      set = calc_mid_sample(item.clone(), prev_min, prev_value, prev_jd, "set");
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
  vec![rise, set, mc, ic].iter().filter(|item| item.jd > 0f64).map(|item| item.clone()).collect::<Vec<AltitudeSample>>()
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
  )
}
