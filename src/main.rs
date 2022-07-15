mod lib;
mod extensions;
mod constants;
mod query_params;
mod routes;
mod help;

extern crate libc;
extern crate serde_derive;
extern crate serde_json;
extern crate libswe_sys;
extern crate chrono;

/* //use libswe_sys::sweconst::{Bodies, Calandar, HouseSystem};
use libswe_sys::sweconst::{
    Bodies, Calandar, OptionalFlag,
}; */
use std::{thread, time};
use libswe_sys::swerust::{
    handler_swe02::*,
};
use serde::{Serialize, Deserialize};
use serde_json::*;
use clap::Parser;
use lib::{core::*, transitions::*, models::{geo_pos::*, graha_pos::*, date_info::*, general::*}, utils::{validators::*, converters::*}, planet_stations::{match_all_planet_stations_range, BodySpeedSet}};
use extensions::swe::{set_sid_mode};
use std::sync::Mutex;
use actix_web::{get, App, HttpServer, Responder, web::{self, Data}};
use std::path::Path;
use lib::julian_date::{current_datetime_string, current_year};
use constants::*;
use query_params::*;
use help::*;
use routes::{chart_data::*, transitions::*};

/// Astrologic engine config
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    // Ephemeris path
    #[clap(short, long, value_parser, default_value_t = SWEPH_PATH_DEFAULT.to_string() )]
    ephemeris: String,
    #[clap(short, long, value_parser, default_value_t = DEFAULT_PORT )]
    port: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct AppData {
  path: String,
}



#[get("/jd/{dateref}")]
async fn date_info(dateref: web::Path<String>) -> impl Responder {
  let date_str = dateref.as_str();
  let info = if is_decimal_str(date_str) { DateInfo::new_from_jd(date_str.parse::<f64>().unwrap()) } else { DateInfo::new(date_str) };
  web::Json(json!(info))
}

#[get("/date")]
async fn date_info_geo(params: web::Query<InputOptions>) -> impl Responder {
  let dateref: String = params.dt.clone().unwrap_or(current_datetime_string());
  let loc: String = params.loc.clone().unwrap_or("0,0".to_string());
  let geo = if let Some(geo_pos) = loc_string_to_geo(loc.as_str()) { geo_pos } else { GeoPos::zero() };
  let date = if is_decimal_str(dateref.as_str()) { DateInfo::new_from_jd(dateref.parse::<f64>().unwrap()) } else { DateInfo::new(dateref.as_str()) };
  let tz_secs =  params.tzs.clone().unwrap_or(0i16);
  let iso_mode: bool = params.iso.clone().unwrap_or(0) > 0;
  let offset_secs = if tz_secs != 0i16 { Some(tz_secs) } else { None };
  let (indian, prev, base, next, calc_offset_secs) = to_indian_time_with_transitions(date.jd, geo, offset_secs, iso_mode);
  web::Json(json!({ "date": date, "indianTime": indian,  "offsetSecs": calc_offset_secs, "sun": { "prev": prev, "current": base, "next": next } }))
}


#[get("/test-geo-start")]
async fn test_geo_start(params: web::Query<InputOptions>) -> impl Responder {
  let loc: String = params.loc.clone().unwrap_or("0,0".to_string());
  let geo = if let Some(geo_pos) = loc_string_to_geo(loc.as_str()) { geo_pos } else { GeoPos::zero() };
  let dateref: String = params.dt.clone().unwrap_or(current_datetime_string());
  let date = if is_decimal_str(dateref.as_str()) { DateInfo::new_from_jd(dateref.parse::<f64>().unwrap()) } else { DateInfo::new(dateref.as_str()) };
  let start_jd = start_jd_geo(date.jd, geo.lng);
  let start = DateInfo::new_from_jd(start_jd);
  web::Json(json!({ "date": date, "dayStart": start, "lng": geo.lng, "lat": geo.lat }))
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct PositionInfo {
  date: DateInfo,
  geo: GeoPos,
  positions: Vec<GrahaPosSet>
}

impl PositionInfo {
  fn new(date: DateInfo, geo: GeoPos, positions: Vec<GrahaPosSet>) -> PositionInfo {
    PositionInfo{ date, geo, positions }
  }
}


#[get("/planet-stations")]
async fn planet_stations_progress(params: web::Query<InputOptions>) -> impl Responder {
  let micro_interval = time::Duration::from_millis(30);  
  let dateref: String = params.dt.clone().unwrap_or(current_datetime_string());
  let def_keys = vec!["me", "ve", "ma", "ju", "sa", "ur", "ne", "pl"];
  let key_string: String = params.bodies.clone().unwrap_or("".to_string());
  let keys = body_keys_str_to_keys_or(key_string, def_keys);
  let date = DateInfo::new(dateref.to_string().as_str());
  let dateref_end: String = params.dt2.clone().unwrap_or(current_datetime_string());
  let iso_mode: bool = params.iso.clone().unwrap_or(0) > 0;
  let end_date = DateInfo::new(dateref_end.to_string().as_str());
  let items: Vec<BodySpeedSet> = match_all_planet_stations_range(date.jd, end_date.jd, to_str_refs(&keys), iso_mode);
  let valid = items.len() > 0;
  thread::sleep(micro_interval);
  web::Json(json!({ "valid": valid, "start": date,  "end": end_date, "items": items }))
}

#[get("/p2")]
async fn progress_synastry(params: web::Query<InputOptions>) -> impl Responder {
  let micro_interval = time::Duration::from_millis(30);  
  let loc: String = params.loc.clone().unwrap_or("0,0".to_string());
  let geo = if let Some(geo_pos) = loc_string_to_geo(loc.as_str()) { geo_pos } else { GeoPos::zero() };
  let dateref: String = params.dt.clone().unwrap_or(current_datetime_string());
  let p2_ago: u8 = params.p2ago.clone().unwrap_or(1);
  let p2_start: u16 = params.p2start.clone().unwrap_or(0);
  let p2_start_year = if p2_start > 1800 { p2_start as u32 } else { current_year() as u32 - p2_ago as u32 };
  let p2_years: u8 = params.p2yrs.clone().unwrap_or(3);
  let p2_per_year: u8 = params.p2py.clone().unwrap_or(2);
  let def_keys = vec!["su", "mo", "ma", "me", "ju", "ve", "sa"];
  let key_string: String = params.bodies.clone().unwrap_or("".to_string());
  let keys = body_keys_str_to_keys_or(key_string, def_keys);
  let date = DateInfo::new(dateref.to_string().as_str());
  let items: Vec<ProgressItemSet> = get_bodies_p2(date.jd, keys, p2_start_year, p2_years as u16, p2_per_year);
  let valid = items.len() > 0;
  thread::sleep(micro_interval);
  web::Json(json!({ "valid": valid, "date": date,  "start_year": p2_start_year, "years": p2_years, "per_year": p2_per_year, "geo": geo, "items": items }))
}

async fn date_now() -> impl Responder {
  web::Json(json!(DateInfo::now()))
}

async fn welcome() -> impl Responder {
  web::Json(json!({ "message": "Welcome to Astro API", "time": DateInfo::now(), "routes": endpoint_help() }))
}

async fn welcome_not_configured() -> impl Responder {
  web::Json( json!({ "valid": false, "message": "Welcome to Astro API", "error": "Incorrect ephemeris path", "time": DateInfo::now() }))
}

async fn route_not_found() -> impl Responder {
  web::Json( json!({ "valid": false, "error": "route not found" }))
}

#[actix_web::main]
async fn main()  -> std::io::Result<()> {
  
    let args = Args::parse();
    let ephemeris_path = args.ephemeris;
    let port = args.port as u16;
    let has_path = Path::new(&ephemeris_path).exists();
    if  has_path {
      set_ephe_path(ephemeris_path.as_str());
      set_sid_mode(0);
    }
    
    let data = Data::new(Mutex::new(AppData{ path: ephemeris_path }));

    HttpServer::new(move || {
      if has_path {
        App::new()
        .app_data(Data::clone(&data))
          .route("/", web::get().to(welcome))
          .route("/jd", web::get().to(date_now))
          .service(date_info)
          .service(date_info_geo)
          .service(test_geo_start)
          .service(bodies_progress)
          .service(body_positions)
          .service(chart_data_flexi)
          .service(progress_synastry)
          .service(list_sun_transitions)
          .service(pheno_data)
          .service(list_transitions)
          .service(test_transitions)
          .service(body_transposed_transitions_range)
          .service(planet_stations_progress)
          .route("/{sec1}", web::get().to(route_not_found))
          .route("/{sec1}/{sec2}", web::get().to(route_not_found))
          .route("/{sec1}/{sec2}/{sec3}", web::get().to(route_not_found))
          .route("/{sec1}/{sec2}/{sec3}/{sec4}", web::get().to(route_not_found))
          .route("/{sec1}/{sec2}/{sec3}/{sec4}/{sec5}", web::get().to(route_not_found))
      } else {
        App::new()
        .app_data(Data::clone(&data))
          .route("/", web::get().to(welcome_not_configured))
          .route("/{sec1}", web::get().to(welcome_not_configured))
          .route("/{sec1}/{sec2}", web::get().to(welcome_not_configured))
          .route("/{sec1}/{sec2}/{sec3}", web::get().to(welcome_not_configured))
          .route("/{sec1}/{sec2}/{sec3}/{sec4}", web::get().to(route_not_found))
          .route("/{sec1}/{sec2}/{sec3}/{sec4}/{sec5}", web::get().to(route_not_found))
      }
  })
  .bind(("127.0.0.1", port))?
  .run()
  .await
}
