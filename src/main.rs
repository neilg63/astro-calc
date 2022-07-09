mod lib;
mod extensions;
mod constants;

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
use lib::{core::*, transposed_transitions::*, transitions::*, models::{geo_pos::*, graha_pos::*, houses::*, date_info::*, general::*}, utils::{validators::*, converters::*}, help::*, settings::ayanamshas::match_ayanamsha_key};
use extensions::swe::{set_sid_mode};
use std::sync::Mutex;
use actix_web::{get, App, HttpServer, Responder, HttpRequest, web::{self, Data}};
use std::path::Path;
use std::collections::{HashMap};
use lib::julian_date::{current_datetime_string, current_year};
use constants::*;

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
  eq: Option<u8>, // 0 = ecliptic, 1 equatorial, 2 both, 3 with pheno data
  ph: Option<u8>, // 0 = none (except via eq=4 in /chart-data), 1 = show pheno data
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
  p2start: Option<u16>, // p2 start year (overrides p2 ago)
  p2py: Option<u8>, // num per year
  p2bodies: Option<String>, // p2 body keys from su, mo, ma, me, ju, ve, sa
  aya: Option<String>, // ayanamshas
  amode: Option<String>, // apply referenced sidereal type (ayanamsha) to all longitudes
  sid: Option<u8>, // 0 tropical longitudes, 1 sidereal longitudes
  hsys: Option<String>, // comma-separated list of letters representing house systems to be returned. Defaults to W for whole house system
  iso: Option<u8>, // 0 show JD, 1 show ISO UTC
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
  let num_samples = days * per_day_f64 as u16; 
  let days_spanned = if num_samples > 1000 { (1000f64 / per_day_f64) as u16 } else { days };
  let micro_interval = time::Duration::from_millis(20 + (num_samples / 4) as u64);
  let keys = body_keys_str_to_keys_or(key_string, def_keys);
  let date = DateInfo::new(dateref.to_string().as_str());
  let geo_opt = if topo { Some(geo) } else { None };
  let data = calc_bodies_positions_jd(date.jd, to_str_refs(&keys), days_spanned, per_day_f64, geo_opt, eq);
  let frequency = if per_day_f64 < 1f64 { format!("{} days", day_span) } else { format!("{} per day", per_day_f64) };
  thread::sleep(micro_interval);
  web::Json(json!(json!({ "date": date, "geo": geo, "items": data, "num_samples": num_samples, "days": days, "frequency": frequency })))
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

#[get("/sun-transitions")]
async fn list_sun_transitions(params: web::Query<InputOptions>) -> impl Responder {
  let micro_interval = time::Duration::from_millis(30);  
  let loc: String = params.loc.clone().unwrap_or("0,0".to_string());
  let geo = if let Some(geo_pos) = loc_string_to_geo(loc.as_str()) { geo_pos } else { GeoPos::zero() };
  let dateref: String = params.dt.clone().unwrap_or(current_datetime_string());
  let days: u16 = params.days.unwrap_or(28);
  let iso_mode: bool = params.iso.clone().unwrap_or(0) > 0;
  let info = DateInfo::new(dateref.to_string().as_str());
  let sun_transitions_jd = calc_transitions_sun(info.jd, days, geo);
  let sun_transitions: Vec<FlexiValue> = sun_transitions_jd.iter().filter(|item| item.value != 0f64).map(|item| item.as_flexi_value(iso_mode)).collect();
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
  let date = DateInfo::new(dateref.to_string().as_str());
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
  web::Json(json!({ "valid": valid, "date": date, "geo": geo, "transitionSets": transition_sets }))
}

#[get("/test-transitions")]
async fn test_transitions(params: web::Query<InputOptions>) -> impl Responder {
  let micro_interval = time::Duration::from_millis(30);
  let loc: String = params.loc.clone().unwrap_or("0,0".to_string());
  let geo = if let Some(geo_pos) = loc_string_to_geo(loc.as_str()) { geo_pos } else { GeoPos::zero() };
  let dateref: String = params.dt.clone().unwrap_or(current_datetime_string());
  let date = DateInfo::new(dateref.to_string().as_str());
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
  web::Json(json!({ "valid": valid, "date": date, "geo": geo, "transitionSets": transition_sets, "altTransitionets": alt_transition_sets }))
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
  
  web::Json(json!({ "valid": valid, "date": date, "geo": geo, "bodies": bodies, "topoVariants": topo_items, "house": house_data, "ayanamshas": ayanamshas, "transitions": transitions, "progressItems": p2, "pheno": pheno_items }))
}

#[get("/pheno")]
async fn pheno_data(params: web::Query<InputOptions>) -> impl Responder {
  let dateref: String = params.dt.clone().unwrap_or(current_datetime_string());
  let def_keys = vec!["su", "mo", "ma", "me", "ju", "ve", "sa"];
  let key_string: String = params.bodies.clone().unwrap_or("".to_string());
  let keys = body_keys_str_to_keys_or(key_string, def_keys);
  let date = DateInfo::new(dateref.to_string().as_str());
  let items =  get_pheno_results(date.jd, to_str_refs(&keys));
  let valid = items.len() > 0;
  web::Json(json!({ "valid": valid, "date": date, "result": items }))
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
          .service(bodies_progress)
          .service(body_positions)
          .service(chart_data_flexi)
          .service(progress_synastry)
          .service(list_sun_transitions)
          .service(pheno_data)
          .service(list_transitions)
          .service(test_transitions)
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
