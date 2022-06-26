mod lib;
mod extensions;

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
use lib::{core::*, transposed_transitions::*, transitions::*, models::{geo_pos::*, graha_pos::*, houses::*, date_info::*, general::*}, utils::{validators::*, converters::*}, help::*};
use extensions::swe::{set_sid_mode};
use std::sync::Mutex;
use actix_web::{get, App, HttpServer, Responder, HttpRequest, web::{self, Data}};
use std::path::Path;
use std::collections::{HashMap};
use lib::julian_date::{current_datetime_string, current_year};

const SWEPH_PATH_DEFAULT: &str = "/usr/share/libswe/ephe";
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

#[derive(Serialize, Deserialize, Debug, Clone)]
struct AppData {
  path: String,
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
  topo: Option<u8>, // 0 = geocentric, 1 topocentric, 2 both, default 0
  eq: Option<u8>, // 0 = ecliptic, 1 equatorial, 2 both default 0
  days: Option<u16>, // duration in days where applicable
  pd: Option<u8>, // number per day, 2 => every 12 hours
  dspan: Option<u8>, // number per days per calculation
  years: Option<u16>, // duration in years where applicable
  loc: Option<String>, // comma-separated lat,lng(,alt) numeric string
  loc2: Option<String>, // comma-separated lat,lng(,alt) numeric string
  body: Option<String>, // primary celestial body key
  ct: Option<u8>, // show current transitions (for transposed transitions and chart-data )
  p2: Option<u8>, // show progress items ( P2 )
  p2ago: Option<u8>, // years ago for P2
  p2yrs: Option<u8>, // num years for p2
  p2py: Option<u8>, // num per year
  aya: Option<String>, // ayanamshas
  amode: Option<String>, // apply referenced sidereal type (ayanamsha) to all longitudes
  sid: Option<u8>, // 0 tropical longitudes, 1 sidereal longitudes
  hsys: Option<String>, // comma-separated list of letters representing house systems to be returned. Defaults to W for whole house system
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

#[get("/progress")]
async fn bodies_progress(params: web::Query<InputOptions>) -> impl Responder {
  let dateref: String = params.dt.clone().unwrap_or(current_datetime_string());
  let loc: String = params.loc.clone().unwrap_or("0,0".to_string());
  let geo = if let Some(geo_pos) = loc_string_to_geo(loc.as_str()) { geo_pos } else { GeoPos::zero() };
  let def_keys = vec!["su", "mo", "ma", "me", "ju", "ve", "sa", "ur", "ne", "pl", "ke"];
  let key_string: String = params.bodies.clone().unwrap_or("".to_string());
  let topo: bool = params.topo.clone().unwrap_or(0) > 0;
  let eq: bool = params.eq.clone().unwrap_or(0)  > 0; // 0 ecliptic, 1 equatorial, 2 both
  let days: u16 = params.days.unwrap_or(28);
  let per_day = params.pd.clone().unwrap_or(0);
  let day_span = params.dspan.clone().unwrap_or(0);
  let per_day_f64 = if per_day > 0 && per_day < 24 { per_day as f64 } else if day_span > 0 && (day_span as u16) < days { 1f64 / day_span as f64 } else { 2f64 };
  let num_cycles = days * per_day_f64 as u16; 
  let days_spanned = if num_cycles > 1000 { (1000f64 / per_day_f64) as u16 } else { days };
  let micro_interval = time::Duration::from_millis(20 + (num_cycles / 4) as u64);
  let keys = body_keys_str_to_keys_or(key_string, def_keys);
  let date = DateInfo::new(dateref.to_string().as_str());
  let geo_opt = if topo { Some(geo) } else { None };
  let data = calc_bodies_positions_jd(date.jd, to_str_refs(&keys), days_spanned, per_day_f64, geo_opt, eq);
  thread::sleep(micro_interval);
  web::Json(json!(PositionInfo::new(date, geo, data)))
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
  let eq: u8 = params.eq.clone().unwrap_or(2); // 0 ecliptic, 1 equatorial, 2 both
  let info = DateInfo::new(dateref.to_string().as_str());
  let ayanamsha = get_ayanamsha_value(info.jd, aya.as_str());
  let aya_offset = if sidereal { ayanamsha } else { 0f64 };
  let longitudes = match eq {
    1 => match topo { 
      1 => get_body_longitudes_eq_topo(info.jd, geo, aya_offset),
      _ => get_body_longitudes_eq_geo(info.jd, geo, aya_offset)
    },
    _ => match topo { 
      1 => get_body_longitudes_topo(info.jd, geo, aya_offset),
      _ => get_body_longitudes_geo(info.jd, geo, aya_offset)
    }
  };
  let valid = longitudes.len() > 0;
  let sun_transitions = calc_transition_sun(info.jd, geo);
  let moon_transitions = calc_transition_moon(info.jd, geo);
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
  web::Json(json!({ "valid": valid, "date": info, "geo": geo, "longitudes": longitudes, "ayanamsha": { "key": aya, "value": ayanamsha, "applied": sidereal }, "coordinateSystem": coord_system, "sunTransitions": sun_transitions, "moonTransitions": moon_transitions }))
}

#[get("/sun-transitions")]
async fn list_sun_transitions(params: web::Query<InputOptions>) -> impl Responder {
  let micro_interval = time::Duration::from_millis(30);  
  let loc: String = params.loc.clone().unwrap_or("0,0".to_string());
  let geo = if let Some(geo_pos) = loc_string_to_geo(loc.as_str()) { geo_pos } else { GeoPos::zero() };
  let dateref: String = params.dt.clone().unwrap_or(current_datetime_string());
  let days: u16 = params.days.unwrap_or(28);
  let info = DateInfo::new(dateref.to_string().as_str());
  let sun_transitions = calc_transitions_sun(info.jd, days, geo);
  let valid = sun_transitions.len() > 0;
  thread::sleep(micro_interval);
  web::Json(json!({ "valid": valid, "date": info, "geo": geo, "sunTransitions": sun_transitions }))
}

#[get("/transitions")]
async fn list_transitions(params: web::Query<InputOptions>) -> impl Responder {
  let micro_interval = time::Duration::from_millis(30);
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
  thread::sleep(micro_interval);
  web::Json(json!({ "valid": valid, "date": info, "geo": geo, "transitionSets": transition_sets }))
}

#[get("/chart-data")]
async fn chart_data_flexi(params: web::Query<InputOptions>) -> impl Responder {
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
  let p2_ago: u8 = params.p2ago.clone().unwrap_or(1);
  let p2_start_year = current_year() as u32 - p2_ago as u32;
  let p2_years: u8 = params.p2yrs.clone().unwrap_or(3);
  let p2_per_year: u8 = params.p2py.clone().unwrap_or(2);
  let def_keys = vec!["su", "mo", "ma", "me", "ju", "ve", "sa", "ur", "ne", "pl"];
  let key_string: String = params.bodies.clone().unwrap_or("".to_string());
  let keys = body_keys_str_to_keys_or(key_string, def_keys);
  let date = DateInfo::new(dateref.to_string().as_str());
  let data = match topo {
    1 => match eq {
      0 => get_bodies_ecl_topo(date.jd, to_str_refs(&keys), geo),
      1 => get_bodies_eq_topo(date.jd, to_str_refs(&keys), geo),
      _ => get_bodies_dual_topo(date.jd, to_str_refs(&keys), geo),
    }
    _ => match eq {
      0 => get_bodies_ecl_geo(date.jd, to_str_refs(&keys)),
      1 => get_bodies_eq_geo(date.jd, to_str_refs(&keys)),
      _ => get_bodies_dual_geo(date.jd, to_str_refs(&keys)),
    }
  };
  let mut topo_items: Vec<LngLatKey> = Vec::new();
  if topo == 2 {
    topo_items = get_bodies_ecl_topo(date.jd, to_str_refs(&keys), geo).into_iter().map(|b| b.to_lng_lat_key()).collect();
  }
  let valid = data.len() > 0;
  let house_data = if match_all_houses { get_all_house_systems(date.jd, geo) } else { get_house_systems(date.jd, geo, h_systems) } ;
  let ayanamshas = match aya.as_str() {
    "all" => get_all_ayanamsha_values(date.jd),
    _ => get_ayanamsha_values(date.jd, aya_keys),
  };
  
  let transitions: Vec<KeyNumValueSet> = if show_transitions { get_transition_sets(date.jd, to_str_refs(&keys), geo) } else { Vec::new() };
  
  let p2: Vec<ProgressItemSet> = if show_p2 { get_bodies_p2(date.jd, keys.clone(), p2_start_year, p2_years as u16, p2_per_year) } else { Vec::new() };

  let bodies: FlexiBodyPos = match eq {
    0 => FlexiBodyPos::Simple(data.iter().map(|b| b.to_body("ecl")).collect()),
    1 => FlexiBodyPos::Simple(data.iter().map(|b| b.to_body("eq")).collect()),
    _=> FlexiBodyPos::Extended(data),
  };
  thread::sleep(micro_interval);
  web::Json(json!({ "valid": valid, "date": date, "geo": geo, "bodies": bodies, "topoVariants": topo_items, "house": house_data, "ayanamshas": ayanamshas, "transitions": transitions, "progressItems": p2 }))
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
async fn body_transposed_transitions_range(params: web::Query<InputOptions>) -> impl Responder {
  let micro_interval = time::Duration::from_millis(50);
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
  thread::sleep(micro_interval);
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
          .route("/ephemeris-path", web::get().to(show_path))
          .route("/jd", web::get().to(date_now))
          .service(date_info)
          .service(bodies_progress)
          .service(body_positions)
          .service(chart_data_flexi)
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
