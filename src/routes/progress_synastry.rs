use std::{thread, time};
use serde_json::*;
use actix_web::{get, Responder,web::{Query, Json} };
use super::super::lib::{julian_date::{current_year}, core::*,  models::{geo_pos::*, general::*}, utils::{converters::*}};
use super::super::{reset_ephemeris_path, query_params::*};

#[get("/p2")]
async fn progress_synastry_items(params: Query<InputOptions>) -> impl Responder {
  reset_ephemeris_path();
  let micro_interval = time::Duration::from_millis(30);  
  let loc: String = params.loc.clone().unwrap_or("0,0".to_string());
  let geo = if let Some(geo_pos) = loc_string_to_geo(loc.as_str()) { geo_pos } else { GeoPos::zero() };
  let date = to_date_object(&params);
  let p2_ago: u8 = params.p2ago.clone().unwrap_or(1);
  let p2_start: u16 = params.p2start.clone().unwrap_or(0);
  let p2_start_year = if p2_start > 1800 { p2_start as u32 } else { current_year() as u32 - p2_ago as u32 };
  let p2_years: u8 = params.p2yrs.clone().unwrap_or(3);
  let p2_per_year: u8 = params.p2py.clone().unwrap_or(2);
  let def_keys = vec!["su", "mo", "ma", "me", "ju", "ve", "sa"];
  let key_string: String = params.bodies.clone().unwrap_or("".to_string());
  let keys = body_keys_str_to_keys_or(key_string, def_keys);
  let items: Vec<ProgressItemSet> = get_bodies_p2(date.jd, keys, p2_start_year, p2_years as u16, p2_per_year);
  let valid = items.len() > 0;
  thread::sleep(micro_interval);
  Json(json!({ "valid": valid, "date": date,  "start_year": p2_start_year, "years": p2_years, "per_year": p2_per_year, "geo": geo, "items": items }))
}
