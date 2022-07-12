
use std::{thread, time};
use serde_json::*;
use super::super::lib::{core::*,  transitions::*, models::{geo_pos::*, graha_pos::*, houses::*, date_info::*, general::*}, utils::{converters::*}, settings::{ayanamshas::{match_ayanamsha_key}}};
use actix_web::{get, Responder,web::{self} };
use super::super::lib::julian_date::{current_datetime_string, current_year};
use super::super::{query_params::*};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
struct ChartDataResult {
  valid: bool,
  date: DateInfo,
  geo: GeoPos,
  bodies: FlexiBodyPos,
  #[serde(rename="topoVariants",skip_serializing_if = "Vec::is_empty")]
  topo_variants: Vec<LngLatKey>,
  house: HouseSetData,
  #[serde(skip_serializing_if = "Vec::is_empty")]
  ayanamshas: Vec<KeyNumValue>,
  #[serde(skip_serializing_if = "Vec::is_empty")]
  transitions: Vec<KeyFlexiValueSet>,
  #[serde(rename="progressItems",skip_serializing_if = "Vec::is_empty")]
  progress_items: Vec<ProgressItemSet>,
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pheno: Vec<PhenoItem>
}



#[get("/positions")]
async fn body_positions(params: web::Query<InputOptions>) -> impl Responder {
  let micro_interval = time::Duration::from_millis(20);
  let dateref: String = params.dt.clone().unwrap_or(current_datetime_string());
  let loc: String = params.loc.clone().unwrap_or("0,0".to_string());
  let geo = if let Some(geo_pos) = loc_string_to_geo(loc.as_str()) { geo_pos } else { GeoPos::zero() };
  let aya: String = params.aya.clone().unwrap_or("true_citra".to_string());
  let sidereal: bool = params.sid.unwrap_or(0) > 0;
  let topo: u8 = params.topo.clone().unwrap_or(0);
  let def_keys =  vec!["su", "mo", "ma", "me", "ju", "ve", "sa", "ur", "ne", "pl", "ra", "ke"];
  let key_string: String = params.bodies.clone().unwrap_or("".to_string());
  let keys = body_keys_str_to_keys_or(key_string, def_keys);
  let eq: u8 = params.eq.clone().unwrap_or(2); // 0 ecliptic, 1 equatorial, 2 both
  let date = DateInfo::new(dateref.to_string().as_str());
  let aya_key = match_ayanamsha_key(aya.as_str());
  let ayanamsha = get_ayanamsha_value(date.jd, aya.as_str());
  let aya_offset = if sidereal { ayanamsha } else { 0f64 };
  let longitudes = match eq {
    1 => match topo { 
      1 => get_body_longitudes_eq_topo(date.jd, geo, aya_offset, to_str_refs(&keys)),
      _ => get_body_longitudes_eq_geo(date.jd, geo, aya_offset, to_str_refs(&keys))
    },
    _ => match topo { 
      1 => get_body_longitudes_topo(date.jd, geo, aya_offset, to_str_refs(&keys)),
      _ => get_body_longitudes_geo(date.jd, geo, aya_offset, to_str_refs(&keys))
    }
  };
  let valid = longitudes.len() > 0;
  let sun_transitions = calc_transition_sun(date.jd, geo);
  let moon_transitions = calc_transition_moon(date.jd, geo);
  let eq_label = match eq {
    1 => "equatorial",
    _ => "ecliptic",
  };
  let topo_label = match topo {
    1 => "topocentric",
    _ => "geocentric",
  };
  let coord_system = format!("{}/{}", eq_label, topo_label );
  thread::sleep(micro_interval);
  web::Json(json!({ "valid": valid, "date": date, "geo": geo, "longitudes": longitudes, "ayanamsha": { "key": aya_key, "value": ayanamsha, "applied": sidereal }, "coordinateSystem": coord_system, "sunTransitions": sun_transitions, "moonTransitions": moon_transitions }))
}

#[get("/chart-data")]
pub async fn chart_data_flexi(params: web::Query<InputOptions>) -> impl Responder {
  let micro_interval = time::Duration::from_millis(50);
  let dateref: String = params.dt.clone().unwrap_or(current_datetime_string());
  let loc: String = params.loc.clone().unwrap_or("0,0".to_string());
  let geo = if let Some(geo_pos) = loc_string_to_geo(loc.as_str()) { geo_pos } else { GeoPos::zero() };
  let show_transitions: bool = params.ct.clone().unwrap_or(0) > 0;
  let aya: String = params.aya.clone().unwrap_or("true_citra".to_string());
  let aya_keys: Vec<&str> = match aya.as_str() {
    "all" => vec![],
    "core" => vec!["true_citra", "lahiri"],
    _ => aya.split(",").collect(),
  };
  let hsys_str = params.hsys.clone().unwrap_or("W".to_string());
  let match_all_houses = hsys_str.to_lowercase().as_str() == "all";
  let h_systems: Vec<char> = if match_all_houses { vec![] } else { match_house_systems_chars(hsys_str) };
  let show_p2: bool = params.p2.clone().unwrap_or(0) > 0;
  let topo: u8 = params.topo.clone().unwrap_or(0);
  let eq: u8 = params.eq.clone().unwrap_or(2); // 0 ecliptic, 1 equatorial, 2 both
  let show_pheno_inline = eq == 3;
  let show_pheno_below = !show_pheno_inline && params.ph.clone().unwrap_or(0) > 0;
  let p2_ago: u8 = params.p2ago.clone().unwrap_or(1);
  let p2_start_year = current_year() as u32 - p2_ago as u32;
  let p2_years: u8 = params.p2yrs.clone().unwrap_or(3);
  let p2_per_year: u8 = params.p2py.clone().unwrap_or(2);
  let def_keys = vec!["su", "mo", "ma", "me", "ju", "ve", "sa", "ur", "ne", "pl"];
  let key_string: String = params.bodies.clone().unwrap_or("".to_string());
  let keys = body_keys_str_to_keys_or(key_string, def_keys);
  let date = DateInfo::new(dateref.to_string().as_str());
  let iso_mode: bool = params.iso.clone().unwrap_or(0) > 0;
  let data = match topo {
    1 => match eq {
      0 => get_bodies_ecl_topo(date.jd, to_str_refs(&keys), geo),
      1 => get_bodies_eq_topo(date.jd, to_str_refs(&keys), geo),
      _ => get_bodies_dual_topo(date.jd, to_str_refs(&keys), geo, show_pheno_inline),
    }
    _ => match eq {
      0 => get_bodies_ecl_geo(date.jd, to_str_refs(&keys)),
      1 => get_bodies_eq_geo(date.jd, to_str_refs(&keys)),
      _ => get_bodies_dual_geo(date.jd, to_str_refs(&keys), show_pheno_inline),
    }
  };
  let pheno_items = if show_pheno_below { get_pheno_results(date.jd, to_str_refs(&keys)) } else { vec![] };
  let mut topo_variants: Vec<LngLatKey> = Vec::new();
  if topo == 2 {
    topo_variants = get_bodies_ecl_topo(date.jd, to_str_refs(&keys), geo).into_iter().map(|b| b.to_lng_lat_key()).collect();
  }
  let valid = data.len() > 0;
  let house = if match_all_houses { get_all_house_systems(date.jd, geo) } else { get_house_systems(date.jd, geo, h_systems) } ;
  let ayanamshas = match aya.as_str() {
    "all" => get_all_ayanamsha_values(date.jd),
    _ => get_ayanamsha_values(date.jd, aya_keys),
  };
  
  let transition_jds: Vec<KeyNumValueSet> = if show_transitions { get_transition_sets(date.jd, to_str_refs(&keys), geo) } else { Vec::new() };
  let transitions: Vec<KeyFlexiValueSet> = transition_jds.iter().map(|item| item.as_flexi_values(iso_mode)).collect();
  let available_p2_keys = vec!["as", "su", "mo", "ma", "me", "ju", "ve", "sa"];
  let p2keys:Vec<String> = keys.clone().iter().filter(|k| available_p2_keys.contains(&k.as_str())).map(|s| s.to_owned()).collect();
  let p2: Vec<ProgressItemSet> = if show_p2 { get_bodies_p2(date.jd, p2keys, p2_start_year, p2_years as u16, p2_per_year) } else { Vec::new() };

  let bodies: FlexiBodyPos = match eq {
    0 => FlexiBodyPos::Simple(data.iter().map(|b| b.to_body("ecl")).collect()),
    1 => FlexiBodyPos::Simple(data.iter().map(|b| b.to_body("eq")).collect()),
    _=> FlexiBodyPos::Extended(data),
  };
  thread::sleep(micro_interval);
  
  //web::Json(json!({ "valid": valid, "date": date, "geo": geo, "bodies": bodies, "topoVariants": topo_variants, "house": house_data, "ayanamshas": ayanamshas, "transitions": transitions, "progressItems": p2, "pheno": pheno_items }))
  web::Json(json!( ChartDataResult{ valid, date, geo, bodies, topo_variants, house, ayanamshas, transitions, progress_items: p2, pheno: pheno_items}))
}