use super::julian_date::*;

const mins_day: i32 = 1440;

pub struct GrahaPos {
  key: String,
  lng: f64,
  lat: f64,
  lng_speed: f64,
  lat_speed: f64,
  rect_ascension: f64,
  declination: f64,
}

impl GrahaPos {

  fn new(key: &str, lng: f64, lat: f64, lng_speed: f64) -> Self {
    GrahaPos { key: key.to_string(), lng: lng, lat, lng_speed: lng_speed }
  }

  fn fixed(key: &str, lng: f64, lat: f64) -> Self {
    GrahaPos { key: key.to_string(), lng: lng, lat: lat, lng_speed: 0f64 }
  }

  fn basic(key: &str, lng: f64) -> Self {
    GrahaPos { key: key.to_string(), lng: lng, lat: 0f64, lng_speed: 0f64 }
  }

}

struct AltitudeSample {
  mode: String,
  mins: f64,
  jd: f64,
  value: f64,
}

impl AltitudeSample {
  
  fn new(mode: &str, mins: f64, jd: f64, value: f64) -> Self {
    AltitudeSample{
      mode: mode.to_string(),
      mins: mins,
      jd: jd,
      value: value,
    }
  }

  fn basic(mode: &str) -> Self {
    AltitudeSample{
      mode: mode.to_string(),
      mins: 0f64,
      jd: 0f64,
      value: 0f64,
    }
  }

  fn iso_date(&self) -> String {
    return if self.jd > 0f64 { julian_day_to_iso_datetime(self.jd) } else { "".to_string() };
  }

  fn set_mode(&self, mode: &str)  {
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
  return AltitudeSample::new(mode, prev_min, mid_point, 0f64);
};

fn recalc_min_max_transit_sample(
  sample: AltitudeSample,
  geo: GeoPos,
  lng: f64,
  lat: f64,
  max_mode: bool,
  multiplier: i8,
) -> AltitudeSample {
  let sample_rate = 0.25f64;
  let num_sub_samples = multiplier as f64 * 2 as f64 * (1f64 / sample_rate);
  let sample_start_jd = sample.jd - num_sub_samples / (2 / sample_rate) / mins_day;
  let sample_start_min = sample.mins - num_sub_samples / (2 / sample_rate);
  let mode = match max_mode { 
    true => "mc",
    false => "ic",
    _ => ""
   };
  let max = num_sub_samples as i32 + 1;
  for i in 0..max {
    let mins = sample_start_min + i * sample_rate;
    let jd = sample_start_jd + (i * sample_rate) / mins_day;
    let value = calc_altitude(jd, geo, lng, lat);
    let item = AltitudeSample::new(mode, mins, jd, value);
    if max_mode && item.value > sample.value {
      sample = item;
    } else if !max_mode && item.value < sample.value {
      sample = item;
    }
  }
  sample
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
  fn match_rise(self) -> bool {
      match &self {
          TransitionFilter::Rise | TransitionFilter::RiseSet | TransitionFilter::All => true,
          _ => false, 
      }
  }

  fn match_set(self) -> bool {
      match &self {
          TransitionFilter::Set | TransitionFilter::RiseSet | TransitionFilter::All => true,
          _ => false, 
      }
  }

  fn match_mc(self) -> bool {
      match &self {
          TransitionFilter::Mc | TransitionFilter::McIc | TransitionFilter::All => true,
          _ => false, 
      }
  }

  fn match_ic(self) -> bool {
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
  sample_key: String,
) -> Vec<AltitudeSample> {
  let max = mins_day / multiplier as i32;
  let items = [];
  let match_set = filter.match_set()
  let match_rise = filter.match_rise();
  let match_mc = filter.match_mc();
  let match_ic = filter.match_ic();
  let mut ic = AltitudeSample::basic("ic");
  let mut rise = AltitudeSample::basic("rise");
  let mut set = AltitudeSample::basic("set");
  let mut mc = AltitudeSample::basic("mc");
  let mut prev_value = 0;
  let mut prev_min = 0;
  let mut prev_jd = 0;
  // resample the longitude and latitude speed for the moon only
  let resample_speed = sample_key == "mo" && lng_speed != 0f64;
  for i in 0..max {
    let n = i as f64 * multiplier as f64;
    let dayFrac = n / mins_day as f64;
    let jd = jd_start + dayFrac;
    let mut sample_spd = lng_speed;
    let latSpd = 0;
    if resample_speed {
      let sample_body = calc_body_jd(jd, sample_key, false, true);
      sample_spd = sample_body.lng_speed;
      latSpd = sample_body.lat_speed;
    }
    let adjusted_lng = lng_speed !== 0 ? lng + sample_spd * dayFrac : lng;
    let adjusted_lat = latSpd !== 0 ? lat + latSpd * dayFrac : lat;
    let value = calc_altitude(jd, geo, adjusted_lng, adjusted_lat);
    let item = AltitudeSample::new('', n,jd, value);
    if (match_mc && value > mc.value) {
      mc = item.set_type("mc");
    }
    if (match_ic && value < ic.value) {
      ic = item.set_type("ic");
    }
    if (match_rise && prev_value < 0 && value > 0) {
      rise = calc_mid_sample(item, prev_min, prev_value, prev_jd, "rise");
    } else if (match_set && prev_value > 0 && value < 0) {
      set = calc_mid_sample(item, prev_min, prev_value, prev_jd, "set");
    }
    if (!match_mc && !match_ic) {
      if (!match_rise && match_set && set.jd > 0) {
        break;
      } else if (!match_set && match_rise && rise.jd > 0) {
        break;
      }
    }
    items.push(item);
    prev_value = value;
    prev_min = n;
    prev_jd = jd;
  }
  if (match_mc && mc.jd > 0) {
    mc = recalc_min_max_transit_sample(mc, geo, lng, lat, true, multiplier);
  }
  if match_ic && ic.jd > 0 {
    ic = recalc_min_max_transit_sample(ic, geo, lng, lat, false, multiplier);
  }
  vec![rise, set, mc, ic].iter().filter(|item| item.jd > 0)
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
    graha_pos.key,
  )
}
