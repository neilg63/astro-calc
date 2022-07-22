use serde::{Deserialize};
use actix_web::{web::{Query} };
use super::lib::{models::date_info::DateInfo, julian_date::{current_datetime_string}};

#[derive(Deserialize)]
pub struct InputOptions {
  pub dt: Option<String>, // primary UTC date string
  pub dtl: Option<String>, // primary date string in local time (requires offset)
  pub jd: Option<f64>, // primary jd as a float
  pub dt2: Option<String>, // secondary UTC date string 
  pub dtl2: Option<String>, // secondary date string in local time (requires offset)
  pub jd2: Option<f64>, // secondary jd as a float
  pub offset: Option<i32>, // offset is seconds from UTC
  pub bodies: Option<String>, // either a comma separated list of required 2-letter celestial body keys or body group keys
  pub topo: Option<u8>, // 0 = geocentric, 1 topocentric, 2 both, default 0
  pub eq: Option<u8>, // 0 = ecliptic, 1 equatorial, 2 both, both 3 with altitude/azimuth, 4 with inline planetary phenomena
  pub ph: Option<u8>, // 0 = none (except via eq=4 in /chart-data), 1 = show pheno(nema) as separate array
  pub days: Option<u16>, // duration in days where applicable
  pub pd: Option<u8>, // number per day, 2 => every 12 hours
  pub dspan: Option<u8>, // number per days per calculation
  pub years: Option<u16>, // duration in years where applicable
  pub loc: Option<String>, // comma-separated lat,lng(,alt) numeric string
  pub loc2: Option<String>, // comma-separated lat,lng(,alt) numeric string
  pub body: Option<String>, // primary celestial body key
  pub ct: Option<u8>, // show current transitions (for transposed transitions and chart-data )
  pub it: Option<u8>, // show indian time divisions based on sunrise/set
  pub p2: Option<u8>, // show progress items ( P2 )
  pub p2ago: Option<u8>, // years ago for P2
  pub p2yrs: Option<u8>, // num years for p2
  pub p2start: Option<u16>, // p2 start year (overrides p2 ago)
  pub p2py: Option<u8>, // num per year
  pub p2bodies: Option<String>, // p2 body keys from su, mo, ma, me, ju, ve, sa
  pub aya: Option<String>, // ayanamshas
  pub amode: Option<String>, // apply referenced sidereal type (ayanamsha) to all longitudes
  pub sid: Option<u8>, // 0 tropical longitudes, 1 sidereal longitudes
  pub hsys: Option<String>, // comma-separated list of letters representing house systems to be returned. Defaults to W for whole house system
  pub retro: Option<u8>, // show planet stations (retrograde, peak), 0 no, 1 yes
  pub iso: Option<u8>, // 0 show JD, 1 show ISO UTC
  pub tzs: Option<i16>, // offset in seconds from UTC
}

pub fn to_ayanamsha_keys(params: &Query<InputOptions>, def_val: &str) -> (Vec<String>, String) {
  let aya: String = params.aya.clone().unwrap_or(def_val.to_string());
  
  let aya_keys = match aya.as_str() {
    "all" => vec![],
    "core" => vec!["true_citra", "lahiri", "krishnamurti"],
    _ => if aya.len() > 1 { aya.as_str().split(",").collect() } else { vec![] },
  }.into_iter().map(|k| k.to_owned()).collect();
  let mode = match aya.as_str() {
    "all"  | "core" => aya,
    _ => "keys".to_string(),
  };
  (aya_keys, mode)
}

pub fn to_date_object_by_num(params: &Query<InputOptions>, num: u8) -> DateInfo {
  let jd = match num {
    2 => params.jd2.clone().unwrap_or(0f64),
    _ => params.jd.clone().unwrap_or(0f64)
  };
  if jd > 1_000_000f64 {
    DateInfo::new_from_jd(jd)
  } else {
    let dateref: String = match num {
      2 => params.dt2.clone().unwrap_or(current_datetime_string()),
      _ => params.dt.clone().unwrap_or(current_datetime_string()),
    };
    DateInfo::new(dateref.to_string().as_str())
  }
}

pub fn to_date_object(params: &Query<InputOptions>) -> DateInfo {
  to_date_object_by_num(&params, 1)
}

pub fn to_date_object_2(params: &Query<InputOptions>) -> DateInfo {
  to_date_object_by_num(&params, 2)
}
