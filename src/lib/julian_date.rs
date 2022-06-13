use chrono::{NaiveDateTime};

pub enum JulianDayEpoch {
  Days = 2440587, // ref year in julian days
  Hours = 12, // ref hours in addition to ref years, 12 hours = 0.5 days
  RefYear = 1970, // ref year at 1 Jan 00:00:00 UTC for conversion from unix time
}

impl JulianDayEpoch {
  fn days_unix() -> f64 {
    JulianDayEpoch::Days as i64 as f64 + JulianDayEpoch::Hours as i64 as f64 / 24f64
  }
}

/**
 * Utility function to convert any ISO-8601-like date string to a Kronos NaiveDateTime object
 * This function accepts YYYY-mm-dd HH:MM:SS separated by a space or letter T and with or without hours, minutes or seconds.
 * Missing time parts will be replaced by 00, hence 2022-06-23 will be 2022-06-23 00:00:00 UTC and 22-06-23 18:20 will be 2022-06-23 18:30:00
 */
pub fn iso_string_to_datetime(dt: &str) -> NaiveDateTime {
  let dt_base = if dt.contains('.') { dt.split(".").next().unwrap() } else { dt };
  let clean_dt = dt_base.replace("T", " ");
  let mut dt_parts = clean_dt.split(" ");
  let date_part = if clean_dt.clone().contains(" ") { dt_parts.next().unwrap().to_string() } else { clean_dt.clone() };
  let time_part = if clean_dt.clone().contains(" ") { dt_parts.next().unwrap().to_string() } else { "".to_string() };
  let mut time_parts = if time_part.len() > 1 { time_part.split(":").into_iter().collect() } else { vec!("00", "00", "00") };
  let num_time_parts = time_parts.len();
  if num_time_parts < 3 { 
    time_parts.push("00");
  }
  if num_time_parts < 2 {
    time_parts.push("00");
  }
  let formatted_str = format!("{} {}:{}:{}", date_part, time_parts[0], time_parts[1], time_parts[2]);
  if let Ok(dt) = NaiveDateTime::parse_from_str(formatted_str.as_str(), "%Y-%m-%d %H:%M:%S") {
    dt
  } else {
    NaiveDateTime::from_timestamp(0, 0)
  }
}

/*
  Convert the current unixtime to julian days
*/
pub fn unixtime_to_julian_day(ts: i64) -> f64 {
  (ts as f64 / 86_400f64) + JulianDayEpoch::days_unix()
}

pub fn datetime_to_julian_day(dt: &str) -> f64 {
  unixtime_to_julian_day(iso_string_to_datetime(dt).timestamp())
}

pub fn julian_day_to_unixtime(jd: f64) -> i64 {
  ((jd - JulianDayEpoch::days_unix() as f64) * 86400f64) as i64
}

pub trait JulianDay {
  fn to_jd(&self) -> f64;
}

impl JulianDay for NaiveDateTime {
    fn to_jd(&self) -> f64 {
      unixtime_to_julian_day(self.timestamp())
    }
}

pub fn julian_day_to_datetime(jd: f64) -> NaiveDateTime {
  NaiveDateTime::from_timestamp(julian_day_to_unixtime(jd), 0)
}

pub fn julian_day_to_iso_datetime(jd: f64) -> String {
  julian_day_to_datetime(jd).format("%Y-%m-%dT%H:%M:%S").to_string()
}