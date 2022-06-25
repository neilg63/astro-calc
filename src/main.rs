mod lib;
mod extensions;

extern crate serde_derive;
extern crate serde_json;
extern crate libswe_sys;
extern crate chrono;

/* //use libswe_sys::sweconst::{Bodies, Calandar, HouseSystem};
use libswe_sys::sweconst::{
    Bodies, Calandar, OptionalFlag,
}; */
use libswe_sys::swerust::{
    handler_swe02::*,
};
use serde::{Serialize, Deserialize};
use serde_json::*;
use clap::Parser;
use lib::{transposed_transitions::*, transitions::*};
use lib::{core::*, models::{geo_pos::*, graha_pos::*, houses::*, date_info::*, general::*}, utils::{validators::*}};
use extensions::swe::{set_sid_mode};
use std::sync::Mutex;
use actix_web::{get, App, HttpServer, Responder, HttpRequest, web::{self, Data}};
use std::path::Path;
use std::collections::{HashMap};
use lib::julian_date::{current_datetime_string};

const SWEPH_PATH_DEFAULT: &str = "/Users/neil/apps/findingyou/findingyou-api/src/astrologic/ephe";
//const SWEPH_PATH_DEFAULT: &str = "/usr/share/libswe/ephe";
const DEFAULT_PORT: u32 = 8087;
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

#[derive(Deserialize, Clone, Debug)]
struct InputOptions {
  dt: Option<String>, // primary UTC date string
  dtl: Option<String>, // primary date string in local time (requires offset)
  jd: Option<f64>, // primary jd as a float
  dt2: Option<String>, // secondary UTC date string 
  dtl2: Option<String>, // secondary date string in local time (requires offset)
  jd2: Option<f64>, // secondary jd as a float
  offset: Option<i32>, // offset is seconds from UTC
  bodies: Option<String>, // either a comma separated list of required 2-letter celestial body keys or body group keys
  topo: Option<u8>, // 0 = geocentrice, 1 topocentric, default 0
  eq: Option<u8>, // 0 = ecliptic, 1 equatorial, 2 both default 0
  days: Option<u16>, // duration in days where applicable
  years: Option<u16>, // duration in years where applicable
  loc: Option<String>, // comma-separated lat,lng(,alt) numeric string
  loc2: Option<String>, // comma-separated lat,lng(,alt) numeric string
  body: Option<String>, // primary celestial body key
  ct: Option<u8>, // show current transitions (for transposed transitions and chart-data )
}


#[derive(Serialize, Deserialize, Debug, Clone)]
struct AppData {
  path: String,
}

fn loc_string_to_geo(loc: &str) -> Option<GeoPos> {
  let parts: Vec<f64> = loc.split(",").into_iter().map(|p| p.parse::<f64>()).filter(|p| match p { Ok(n) => true, _ => false } ).map(|p| p.unwrap()).collect();
  if parts.len() >= 2 {
    let alt = if parts.len() > 2 { parts[2] } else { 0f64 };
    Some(GeoPos::new(parts[0], parts[1], alt))
  } else {
    None
  }
}

fn body_keys_str_to_keys(key_string: String) -> Vec<String> {
  key_string.split(",").into_iter().filter(|p| p.len() == 2).map(|p| p.to_string()).collect()
}

fn body_keys_str_to_keys_or(key_string: String, default_keys: Vec<&str>) -> Vec<String> {
  let keys: Vec<String> = body_keys_str_to_keys(key_string);
  if keys.len() > 0 { keys } else { default_keys.into_iter().map(|p| p.to_string() ).collect() }
}

#[get("/jd/{dateref}")]
async fn date_info(dateref: web::Path<String>) -> impl Responder {
  let date_str = dateref.as_str();
  let info = if is_decimal_str(date_str) { DateInfo::new_from_jd(date_str.parse::<f64>().unwrap()) } else { DateInfo::new(date_str) };
  web::Json(json!(info))
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

#[get("/progress/{dateref}/{loc}")]
async fn bodies_progress(req: HttpRequest) -> impl Responder {
  let dateref: String = req.match_info().get("dateref").unwrap().parse().unwrap();
  let loc: String = req.match_info().query("loc").parse().unwrap();
  let keys = vec!["su", "mo", "ma", "me", "ju", "ve", "sa", "ur", "ne", "pl", "ke"];
  let info = DateInfo::new(dateref.to_string().as_str());
  let parts: Vec<f64> = loc.split(",").into_iter().map(|p| p.parse::<f64>().unwrap()).collect();
  let geo = GeoPos::new(parts[0], parts[1], 0f64);
  let data = calc_bodies_positions_jd_geo(info.jd, keys, 30, 2f64);
  web::Json(json!(PositionInfo::new(info, geo, data)))
}

#[get("/positions/{dateref}/{loc}")]
async fn body_positions(req: HttpRequest) -> impl Responder {
  let dateref: String = req.match_info().get("dateref").unwrap().parse().unwrap();
  let loc: String = req.match_info().query("loc").parse().unwrap();
  let geo = if let Some(geo_pos) = loc_string_to_geo(loc.as_str()) { geo_pos } else { GeoPos::zero() };
  let info = DateInfo::new(dateref.to_string().as_str());
  let longitudes = get_body_longitudes_geo(info.jd, geo);
  let valid = longitudes.len() > 0;
  let ayanamsha = get_ayanamsha_value(info.jd, "true_citra");
  let sun_transitions = calc_transition_sun(info.jd, geo);
  web::Json(json!({ "valid": valid, "date": info, "geo": geo, "longitudes": longitudes, "true_citra": ayanamsha, "sunTransitions": sun_transitions }))
}

#[get("/sun-transitions/{dateref}/{loc}/{num_days}")]
async fn list_sun_transitions(req: HttpRequest) -> impl Responder {
  let dateref: String = req.match_info().get("dateref").unwrap().parse().unwrap();
  let loc: String = req.match_info().query("loc").parse().unwrap();
  let days_ref: String = req.match_info().query("num_days").parse().unwrap();
  let days = if is_integer_str(days_ref.as_str()) { days_ref.parse::<i32>().unwrap() } else { 31i32 };
  let geo = if let Some(geo_pos) = loc_string_to_geo(loc.as_str()) { geo_pos } else { GeoPos::zero() };
  let info = DateInfo::new(dateref.to_string().as_str());
  
  let sun_transitions = calc_transitions_sun(info.jd, days, geo);
  let valid = sun_transitions.len() > 0;
  web::Json(json!({ "valid": valid, "date": info, "geo": geo, "sunTransitions": sun_transitions }))
}

#[get("/transitions")]
async fn list_transitions(req: HttpRequest, params: web::Query<InputOptions>) -> impl Responder {
  let loc: String = params.loc.clone().unwrap_or("0,0".to_string());
  let geo = if let Some(geo_pos) = loc_string_to_geo(loc.as_str()) { geo_pos } else { GeoPos::zero() };
  let dateref: String = params.dt.clone().unwrap_or(current_datetime_string());
  let info = DateInfo::new(dateref.to_string().as_str());
  let def_keys = vec!["su", "mo", "ma", "me", "ju", "ve", "sa"];
  let key_string: String = params.bodies.clone().unwrap_or("".to_string());
  let keys = body_keys_str_to_keys_or(key_string, def_keys);
  let days_int = params.days.unwrap_or(1u16);
  let num_days = if days_int >= 1 { days_int } else { 1u16 };
  let transition_sets = get_transition_sets_extended(info.jd, keys, geo, num_days);
  let valid = transition_sets.len() > 0;
  web::Json(json!({ "valid": valid, "date": info, "geo": geo, "transitionSets": transition_sets }))
}

#[get("/chart-data/{dateref}/{loc}")]
async fn chart_data(req: HttpRequest) -> impl Responder {
  let dateref: String = req.match_info().get("dateref").unwrap().parse().unwrap();
  let loc: String = req.match_info().query("loc").parse().unwrap();
  let geo = if let Some(geo_pos) = loc_string_to_geo(loc.as_str()) { geo_pos } else { GeoPos::zero() };
  let keys = vec!["su", "mo", "ma", "me", "ju", "ve", "sa", "ur", "ne", "pl", "ke"];
  let info = DateInfo::new(dateref.to_string().as_str());
  let data = get_bodies_dual_geo(info.jd, keys.clone());
  let valid = data.len() > 0;
  let house_data = get_all_house_systems(info.jd, geo);
  let ayanamshas = get_all_ayanamsha_values(info.jd);
  //let ayanamshas = get_ayanamsha_value(info.jd, "true_citra");
  let transitions = get_transition_sets(info.jd, keys, geo);
  web::Json(json!({ "valid": valid, "date": info, "geo": geo, "bodies": data, "house": house_data, "ayanamshas": ayanamshas, "transitions": transitions }))
}

#[get("/pheno/{body}/{dateref}")]
async fn pheno_data(req: HttpRequest) -> impl Responder {
  let dateref: String = req.match_info().get("dateref").unwrap().parse().unwrap();
  let body_key: String = req.match_info().query("body").parse().unwrap();
  let info = DateInfo::new(dateref.to_string().as_str());
  let result = get_pheno_result(info.jd, body_key.as_str(), 0i32);
  let valid = result.phase_illuminated != 0f64; 
  web::Json(json!({ "valid": valid, "date": info, "result": result }))
}

#[get("/transposed-transitions")]
async fn body_transposed_transitions_range(req: HttpRequest, params: web::Query<InputOptions>) -> impl Responder {
  let dateref: String = params.dt2.clone().unwrap_or(current_datetime_string());
  let loc: String = params.loc2.clone().unwrap_or("0,0".to_string());
  let historic_geo = if let Some(geo_pos) = loc_string_to_geo(loc.as_str()) { geo_pos } else { GeoPos::zero() };
  let current_loc: String = params.loc.clone().unwrap_or("0,0".to_string());
  let current_geo = if let Some(geo_pos) = loc_string_to_geo(current_loc.as_str()) { geo_pos } else { GeoPos::zero() };
  let dateref_current: String = params.dt.clone().unwrap_or(current_datetime_string());
  let show_transitions: bool = params.ct.clone().unwrap_or(0) > 0;
  let def_keys = vec!["su", "mo", "ma", "me", "ju", "ve", "sa"];
  let key_string: String = params.bodies.clone().unwrap_or("".to_string());
  let keys = body_keys_str_to_keys_or(key_string, def_keys);
  let days_int = params.days.unwrap_or(1u16);
  let num_days = if days_int >= 1 { days_int } else { 1u16 };
  let historic_dt = DateInfo::new(dateref.to_string().as_str());
  let current_dt = DateInfo::new(dateref_current.to_string().as_str());
  let transitions = calc_transposed_graha_transitions_from_source_refs_geo(current_dt.jd, current_geo, historic_dt.jd, historic_geo, keys.clone(), num_days);
  let valid = transitions.len() > 0;
  let current_transitions:  Vec<KeyNumValueSet> = if show_transitions { get_transition_sets_extended(current_dt.jd, keys, current_geo, num_days) } else { Vec::new() };
  web::Json(json!({ "valid": valid, "date": current_dt, "geo": current_geo, "historicDate": historic_dt, "historicGeo": historic_geo, "days": num_days, "transposedTransitions": transitions, "currentTransitions": current_transitions }))
}

async fn date_now() -> impl Responder {
  web::Json(json!(DateInfo::now()))
}

async fn show_path(req: HttpRequest, ) -> impl Responder {
  if let Some(app_data) = req.app_data::<AppData>() {
    web::Json(json!(app_data))
  } else {
    web::Json(json!({ "path": "N/A" }))
  }
}

fn key_num_values_to_map(items: Vec<KeyNumValue>) -> HashMap<String, f64> {
  let mut mp: HashMap<String, f64> = HashMap::new();
  for item in items {
    mp.insert(item.key, item.value);
  }
  mp
}

async fn welcome() -> impl Responder {
  web::Json( json!({ "message": "Welcome to Astro API", "time": DateInfo::now() }))
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
          .route("/ephemeris-path", web::get().to(show_path))
          .route("/jd", web::get().to(date_now))
          .service(date_info)
          .service(bodies_progress)
          .service(body_positions)
          .service(chart_data)
          .service(list_sun_transitions)
          .service(pheno_data)
          .service(list_transitions)
          .service(body_transposed_transitions_range)
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
