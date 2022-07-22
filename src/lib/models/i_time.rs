use chrono::*;
use ::serde::{Serialize, Deserialize};
use super::super::{julian_date::{julian_day_to_datetime}};

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
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

  pub fn new(ref_jd: f64, prev_rise_jd: f64, rise_jd: f64, set_jd: f64, next_rise_jd: f64, start_mode: i8, offset_secs: i16) -> ITime {
    let prev_start = ref_jd < rise_jd;
    let day_before = prev_start && start_mode == 0;
    let is_day_time = (!day_before && ref_jd < set_jd && start_mode != -1) || start_mode == 1;
    let day_start = if prev_start { prev_rise_jd } else { rise_jd };
    let day_length = if prev_start { rise_jd - prev_rise_jd } else { next_rise_jd - rise_jd };
    let offset_jd = offset_secs as f64 / 86400f64;
    let day_before_offset = if day_before { -1f64 } else { 0f64 };
    // chrono date only used to calculate day of the year and week day.
    let dt = julian_day_to_datetime(ref_jd + offset_jd + day_before_offset);
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
