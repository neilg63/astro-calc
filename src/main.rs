mod lib;
mod extensions;

extern crate serde_derive;
extern crate serde_json;
extern crate libswe_sys;  
extern crate ordered_float;
extern crate chrono;
extern crate clap;

//use libswe_sys::sweconst::{Bodies, Calandar, HouseSystem};
use libswe_sys::sweconst::{
    Bodies, Calandar, OptionalFlag,
};
use libswe_sys::swerust::{
    handler_swe02::*, handler_swe03::*, handler_swe08::*, handler_swe14::*,
};
use serde::{Serialize, Deserialize};
use std::fmt;
use clap::{Arg, App};
use chrono::{ NaiveDateTime };
use lib::julian_date::*;
use lib::utils::minmax::*;
use lib::settings::{ayanamshas::*,graha_values::*};
use extensions::swe::*;
use lib::transposed_transitions::*;
use lib::core::*;
use lib::models::geo_pos::*;

const SWEPH_PATH_DEFAULT: &str = "/Users/neil/apps/findingyou/findingyou-api/src/astrologic/ephe";

#[derive(Serialize, Deserialize, Debug)]
pub struct KeyNumValue {
    key: String,
    value: f64,
}

impl KeyNumValue {
    pub fn new(key: &str, value: f64) -> KeyNumValue {
        KeyNumValue { key: key.to_string(), value: value }
    }

    pub fn update(&mut self, value: f64) {
        self.value = value;
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub enum PreferenceValue {
    Numeric(f64),
    String(String),
    None,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Preference {
    key: String,
    value: PreferenceValue,
}

impl Preference {
    fn new_f64(key: &str,value: f64) -> Self {
        Preference{ key: key.to_string(), value: PreferenceValue::Numeric(value) }
    }

    fn new_string(key: &str, value: &str) -> Self {
        Preference{ key: key.to_string(), value: PreferenceValue::String(value.to_string()) }
    }
}

impl fmt::Display for Preference {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.value {
            PreferenceValue::Numeric(val) => write!(f, "{}", val),
            PreferenceValue::String(val) => write!(f, "{}", val),
            _ => write!(f, "")
        }
    }
}

/* pub fn calc_body_jd(jd: f64, key: &str, sideral: bool, topo: bool,
) -> GrahaPos {
  let data: any = {};
  const body = grahaValues.find(b => b.key === key);
  if (body) {
    const topoFlag = topoMode ? swisseph.SEFLG_TOPOCTR : 0;
    const gFlag = sideralMode
      ? swisseph.SEFLG_SIDEREAL | topoFlag
      : swisseph.SEFLG_SWIEPH | swisseph.SEFLG_SPEED | topoFlag;
    await calcUtAsync(jd, body.num, gFlag).catch(result => {
      if (result instanceof Object) {
        result.valid = !result.error;
        processBodyResult(result, body);
        data = {
          num: body.num,
          name: body.subkey,
          friends: body.friends,
          ...result,
        };
      }
    });
  }
  return new Graha(data);
}; */


fn main() {
  
    let matches = App::new("AstroApi")
    .version("1.0")
    .author("Neil Gardner <neilgardner1963@gmail.com>")
    .about("Astrological calculations via Swiss Ephemeris")
    .arg(
      Arg::new("path")
      .short('p')
      .long("path")
      .value_name("path")
      .help("Set the path to the Ephemeris data files")
    )
    .get_matches();
  
  let ephemeris_path = matches.value_of("path").unwrap_or(SWEPH_PATH_DEFAULT);

  set_ephe_path(ephemeris_path);
  let julian_day_ut = julday(1991, 10, 13, 20.0, Calandar::Gregorian);
  println!("13/10/1991 at 20:00 is {}", julian_day_ut);



  let combo = OptionalFlag::Speed as i32 | OptionalFlag::SideralPosition as i32;
  
  let ju_data = calc_ut(julian_day_ut, Bodies::Jupiter, combo);
  println!("Jupiter on 13/10/1991 at 20:00 is at {:?}", ju_data);

  println!("Speed {:?}, SideralPosition {:?}, combo {:?}", OptionalFlag::Speed as i32, OptionalFlag::SideralPosition as i32, combo);

  let house_info: HousesResult = houses(julian_day_ut,56.1f64, -3.4f64, 'W');
  println!("house data {:?}", house_info);

  let house_data: HouseData = HouseData::new(julian_day_ut, 56.1f64, -3.4f64, 'P');
  println!("house data {:?}", house_data);

  let ascendant: f64 = get_ascendant(julian_day_ut, 45.1, 11.7);
  println!("ascendant {:?}", ascendant);

  // let trans_data = rise_trans(julian_day_ut,);
  let trans_data = next_rise(julian_day_ut,Bodies::Sun, 56.1f64, -3.4f64);
  println!("transit jd {:?}", trans_data);

  let start_jd = start_jd_geo(julian_day_ut, 15f64);
  println!("currjd: {} start jd {}", julian_day_ut, start_jd);

  let transit_set = calc_transit_set(julian_day_ut, Bodies::Sun,  56.1f64, -3.4f64);
  println!("transits: {:?}", transit_set);


  let aya = get_ayanamsha(julian_day_ut, Ayanamsha::TrueCitra);
  println!("True citra: {:?}", aya);

  let azalt = calc_altitude(julian_day_ut, true,  56.1f64, -3.4f64, 272.2, 4.111);
  println!("altitude : {:?}", azalt);

  let tt = TransitionFilter::McIc;
  println!("may rise : {}", tt.match_rise());


  let houses: Vec<f64> = vec![178.393837, 202.3903873, 237.30383, 270.3938, 304.20272, 328.3373, 0.123, 32.30383, 57.38363, 94.29272, 119.202827, 152.2028];
  let len = houses.len();
  let min_val: f64 = min_f64(houses.clone());
  println!("min lng : {}, max: {}, {:?}", min_val, max_f64(houses.clone()), houses);

  let ayas = vec![KeyNumValue::new("true_citra", 23.9044764), KeyNumValue::new("lahiri", 24.1054767)];

  let mut aya_val = 0f64;

  if let Some(row) = ayas.iter().find(|a| a.key == "lahiri".to_string()) {
      aya_val = row.value;
  }

  println!("lahari: {}", aya_val);

  let fav_food = Preference::new_string("favourite_food", "Pizza");

  let weight = Preference::new_f64("weight", 74.232938);

  println!("fav food: {}, weight: {}", fav_food, weight);

  let str1 = "Cabbage_Soup_Price";

  let str2 = str1.to_lowercase().replace("_", "").replace(" ", "");

  println!("{}", str2);


  let dt = NaiveDateTime::from_timestamp(31_250_000 * 50, 0);

  println!("{}", dt.format("%Y-%m-%d %H:%M"));

  let precision_val = 10f64 / 3f64;

  println!("10/3: {}", precision_val);

  let approx_val = 10f32 / 13f32;

  println!("approx: 10/13: {}", approx_val);

  let dts: Vec<&str> = vec!("2022-06-09T12:45:32.000Z", "1958-11-23 08:45:32", "1988-03-23 21:25", "1993-09-25 10:00");

  for dt_string in dts {
    let dt = iso_string_to_datetime(dt_string);
    println!("{}: {} / {}", dt_string, dt.timestamp() / 1_000_000, dt.to_jd() );
  }

  let mut kv = KeyNumValue::new("frequency", 20_000f64);
  kv.update(19_000f64);
  println!("{:?}", kv);


  let tc = Ayanamsha::TrueCitra;

  println!("{}: {}", tc, tc.as_i32());

  let max = 13;

  for n in 0..max {
    println!("{}", n);
  }

  let geo = GeoPos::new(56.1, -3.4, 100.0);

  let gr = calc_body_jd(julian_day_ut, "sa", false, false);

  println!("sa: {:?}", gr);
  let curr_jd = current_jd();
  let tr_pos = calc_transposed_graha_transition(curr_jd,geo, gr, TransitionFilter::All, 5);

  println!("curr jd: {}, transitions: {:?}", curr_jd, tr_pos);


  let topo_result = calc_body_jd_topo(julian_day_ut, "me", geo);

  println!("topo me: {:?}", topo_result);

}
