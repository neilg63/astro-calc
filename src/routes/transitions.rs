use std::{thread, time};
use serde_json::*;
use std::collections::HashMap;
use super::super::lib::{traits::{FromKey},transitions::*, transposed_transitions::{calc_transposed_graha_transitions_from_source_refs_topo, calc_transposed_graha_transitions_from_source_refs_geo}, models::{geo_pos::*, general::*}, utils::{converters::*}};
use actix_web::{get, post, Responder,web::{Query, Json}};
use super::super::{query_params::*, reset_ephemeris_path, post_params::*};
use libswe_sys::sweconst::{Bodies};

#[get("/transitions")]
async fn list_transitions(params: Query<InputOptions>) -> impl Responder {
  reset_ephemeris_path();
  let micro_interval = time::Duration::from_millis(30);
  let loc: String = params.loc.clone().unwrap_or("0,0".to_string());
  let geo = if let Some(geo_pos) = loc_string_to_geo(loc.as_str()) { geo_pos } else { GeoPos::zero() };
  let date = to_date_object(&params);
  let def_keys = vec!["su", "mo", "ma", "me", "ju", "ve", "sa"];
  let key_string: String = params.bodies.clone().unwrap_or("".to_string());
  let iso_mode: bool = params.iso.clone().unwrap_or(0) > 0;
  let keys = body_keys_str_to_keys_or(key_string, def_keys);
  let days_int = params.days.unwrap_or(1u16);
  let num_days = if days_int >= 1 { days_int } else { 1u16 };
  let transition_sets_jd = get_transition_sets_extended(date.jd, keys, geo, num_days);
  let valid = transition_sets_jd.len() > 0;
  let transition_sets = FlexiValueSet::FlexiValues(transition_sets_jd.iter().map(|vs| vs.as_flexi_values(iso_mode)).collect());
  thread::sleep(micro_interval);
  Json(json!({ "valid": valid, "date": date, "geo": geo, "transitionSets": transition_sets }))
}

#[get("/sun-transitions")]
async fn list_sun_transitions(params: Query<InputOptions>) -> impl Responder {
  reset_ephemeris_path();
  let micro_interval = time::Duration::from_millis(30);  
  let loc: String = params.loc.clone().unwrap_or("0,0".to_string());
  let geo = if let Some(geo_pos) = loc_string_to_geo(loc.as_str()) { geo_pos } else { GeoPos::zero() };
  let date = to_date_object(&params);
  let days: u16 = params.days.unwrap_or(28);
  let iso_mode: bool = params.iso.clone().unwrap_or(0) > 0;
  let sun_transitions_jd = calc_transitions_sun(date.jd, days, geo);
  let sun_transitions: Vec<FlexiValue> = sun_transitions_jd.iter().filter(|item| item.value != 0f64).map(|item| item.as_flexi_value(iso_mode)).collect();
  let valid = sun_transitions.len() > 0;
  thread::sleep(micro_interval);
  Json(json!({ "valid": valid, "date": date, "geo": geo, "sunTransitions": sun_transitions }))
}

#[get("/pheno")]
async fn pheno_data(params: Query<InputOptions>) -> impl Responder {
  reset_ephemeris_path();
  let def_keys = vec!["su", "mo", "ma", "me", "ju", "ve", "sa"];
  let key_string: String = params.bodies.clone().unwrap_or("".to_string());
  let keys = body_keys_str_to_keys_or(key_string, def_keys);
  let date = to_date_object(&params);
  let items =  get_pheno_results(date.jd, to_str_refs(&keys));
  let valid = items.len() > 0;
  Json(json!({ "valid": valid, "date": date, "result": items }))
}

#[get("/transposed-transitions")]
async fn body_transposed_transitions_range(params: Query<InputOptions>) -> impl Responder {
  reset_ephemeris_path();
  let micro_interval = time::Duration::from_millis(50);
  /* let dateref: String = params.dt2.clone().unwrap_or(current_datetime_string());
  let historic_dt = DateInfo::new(dateref.to_string().as_str()); */
  let historic_dt = to_date_object_2(&params);
  let current_dt = to_date_object(&params);
  let loc: String = params.loc2.clone().unwrap_or("0,0".to_string());
  let historic_geo = if let Some(geo_pos) = loc_string_to_geo(loc.as_str()) { geo_pos } else { GeoPos::zero() };
  let current_loc: String = params.loc.clone().unwrap_or("0,0".to_string());
  let current_geo = if let Some(geo_pos) = loc_string_to_geo(current_loc.as_str()) { geo_pos } else { GeoPos::zero() };
  let show_transitions: bool = params.ct.clone().unwrap_or(0) > 0;
  let def_keys = vec!["su", "mo", "ma", "me", "ju", "ve", "sa"];
  let key_string: String = params.bodies.clone().unwrap_or("".to_string());
  let keys = body_keys_str_to_keys_or(key_string, def_keys);
  let days_int = params.days.unwrap_or(1u16);
  let num_days = if days_int >= 1 { days_int } else { 1u16 };
  let transitions = calc_transposed_graha_transitions_from_source_refs_geo(current_dt.jd, current_geo, historic_dt.jd, historic_geo, keys.clone(), num_days);
  let valid = transitions.len() > 0;
  let current_transitions:  Vec<KeyNumValueSet> = if show_transitions { get_transition_sets_extended(current_dt.jd, keys, current_geo, num_days) } else { Vec::new() };
  thread::sleep(micro_interval);
  Json(json!({ "valid": valid, "date": current_dt, "geo": current_geo, "historicDate": historic_dt, "historicGeo": historic_geo, "days": num_days, "transposedTransitions": transitions, "currentTransitions": current_transitions }))
}

#[post("/transposed-transitions-chart")]
async fn body_transposed_transitions_from_chart(mut payload: Json<PostOptions>) -> impl Responder {
  reset_ephemeris_path();
  let micro_interval = time::Duration::from_millis(50);
  /* let dateref: String = params.dt2.clone().unwrap_or(current_datetime_string());
  let historic_dt = DateInfo::new(dateref.to_string().as_str()); */
  /* let historic_dt = to_date_object_2(&params);
  let current_dt = to_date_object(&params);
  let current_loc: String = params.loc.clone().unwrap_or("0,0".to_string());
  let current_geo = if let Some(geo_pos) = loc_string_to_geo(current_loc.as_str()) { geo_pos } else { GeoPos::zero() };
  let show_transitions: bool = params.ct.clone().unwrap_or(0) > 0;
  let def_keys = vec!["su", "mo", "ma", "me", "ju", "ve", "sa"];
  let key_string: String = params.bodies.clone().unwrap_or("".to_string());
  let keys = body_keys_str_to_keys_or(key_string, def_keys);
  let days_int = params.days.unwrap_or(1u16);
  let num_days = if days_int >= 1 { days_int } else { 1u16 };
  let transitions = calc_transposed_graha_transitions_from_source_refs_geo(current_dt.jd, current_geo, historic_dt.jd, historic_geo, keys.clone(), num_days);
  let valid = transitions.len() > 0;
  let current_transitions:  Vec<KeyNumValueSet> = if show_transitions { get_transition_sets_extended(current_dt.jd, keys, current_geo, num_days) } else { Vec::new() };
  thread::sleep(micro_interval);
  Json(json!({ "valid": valid, "date": current_dt, "geo": current_geo, "historicDate": historic_dt, "historicGeo": historic_geo, "days": num_days, "transposedTransitions": transitions, "currentTransitions": current_transitions })) */
  let params = payload.into_inner();
  let bodies = params.clone().bodies.unwrap_or(vec![]);
  let lngs = params.clone().lngs.unwrap_or(vec![]);
  let positions: HashMap<String, f64> = params.positions.unwrap_or(HashMap::new());
  thread::sleep(micro_interval);
  Json(json!({ "valid": false, "bodies": bodies, "lngs": lngs, "positions": positions }))
}

#[get("/test-transitions")]
async fn test_transitions(params: Query<InputOptions>) -> impl Responder {
  reset_ephemeris_path();
  let micro_interval = time::Duration::from_millis(30);
  let loc: String = params.loc.clone().unwrap_or("0,0".to_string());
  let geo = if let Some(geo_pos) = loc_string_to_geo(loc.as_str()) { geo_pos } else { GeoPos::zero() };
  let date = to_date_object(&params);
  let iso_mode: bool = params.iso.clone().unwrap_or(0) > 0;
  let def_keys = vec!["su", "mo", "ma", "me", "ju", "ve", "sa"];
  let key_string: String = params.bodies.clone().unwrap_or("".to_string());
  let keys = body_keys_str_to_keys_or(key_string, def_keys);
  let days_int = params.days.unwrap_or(1u16);
  let num_days = if days_int >= 1 { days_int } else { 1u16 };
  let transition_sets_jd = get_transition_sets_extended(date.jd, keys.clone(), geo, num_days);
  let valid = transition_sets_jd.len() > 0;
  let transition_sets = FlexiValueSet::FlexiValues(transition_sets_jd.iter().map(|vs| vs.as_flexi_values(iso_mode)).collect());
  let alt_transition_sets_jd = calc_transposed_graha_transitions_from_source_refs_topo(date.jd, geo, date.jd, geo, keys.clone(), num_days);
  let alt_transition_sets = FlexiValueSet::FlexiValues(alt_transition_sets_jd.iter().map(|vs| vs.as_flexi_values(iso_mode)).collect());
  thread::sleep(micro_interval);
  Json(json!({ "valid": valid, "date": date, "geo": geo, "transitionSets": transition_sets, "altTransitionSets": alt_transition_sets }))
}

#[get("/test-swe-mc")]
async fn test_mcs(params: Query<InputOptions>) -> impl Responder {
  reset_ephemeris_path();
  let micro_interval = time::Duration::from_millis(30);
  let loc: String = params.loc.clone().unwrap_or("0,0".to_string());
  let geo = if let Some(geo_pos) = loc_string_to_geo(loc.as_str()) { geo_pos } else { GeoPos::zero() };
  let date = to_date_object(&params);
  let def_keys = vec!["su", "mo", "ma", "me", "ju", "ve", "sa"];
  let key_string: String = params.bodies.clone().unwrap_or("".to_string());
  let keys = body_keys_str_to_keys_or(key_string, def_keys);
  let mut mcs: Vec<KeyNumValue> = vec![];
  let mut num_valid: usize = 0;
  for key in keys {
    let mc = next_mc(date.jd, Bodies::from_key(key.as_str()), geo.lat, geo.lng);
    mcs.push(KeyNumValue::new(key.as_str(), mc));
    if mc >= 0f64 { 
      num_valid += 1;
    }
  }
  let num_items = mcs.len();
  let valid = num_valid == num_items && num_items > 0;
  let desc = "Tests the native Swiss Ephemeris implementation with MC/IC flags, known to be buggy on some platforms. In production, mid point between rise and set is used. Where an object does not set or rise, the MC and IC are calculated by sampling max and min altitdues.";
  thread::sleep(micro_interval);
  Json(json!({ "valid": valid, "description": desc, "date": date, "geo": geo, "values": mcs }))
}