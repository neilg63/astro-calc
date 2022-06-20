mod lib;
mod extensions;

extern crate serde_derive;
extern crate serde_json;
extern crate libswe_sys;  
extern crate ordered_float;
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

fn loc_string_to_geo(loc: &str) -> Option<GeoPos> {
  let parts: Vec<f64> = loc.split(",").into_iter().map(|p| p.parse::<f64>()).filter(|p| match p { Ok(n) => true, _ => false } ).map(|p| p.unwrap()).collect();
  if parts.len() >= 2 {
    let alt = if parts.len() > 2 { parts[2] } else { 0f64 };
    Some(GeoPos::new(parts[0], parts[1], alt))
  } else {
    None
  }
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

#[get("/transposed-transitions/{current_date}/{current_loc}/{historic_date}/{historic_loc}")]
async fn body_transposed_transitions(req: HttpRequest) -> impl Responder {
  let dateref: String = req.match_info().get("historic_date").unwrap().parse().unwrap();
  let loc: String = req.match_info().query("historic_loc").parse().unwrap();
  let historic_geo = if let Some(geo_pos) = loc_string_to_geo(loc.as_str()) { geo_pos } else { GeoPos::zero() };
  let current_loc: String = req.match_info().query("current_loc").parse().unwrap();
  let current_geo = if let Some(geo_pos) = loc_string_to_geo(current_loc.as_str()) { geo_pos } else { GeoPos::zero() };
  let dateref_current: String = req.match_info().get("current_date").unwrap().parse().unwrap();
  let keys = vec!["su", "mo", "ma", "me", "ju", "ve", "sa", "ke"];
  let historic_dt = DateInfo::new(dateref.to_string().as_str());
  let current_dt = DateInfo::new(dateref_current.to_string().as_str());
  let transitions = calc_transposed_graha_transitions_from_source_refs_geo(current_dt.jd, current_geo, historic_dt.jd, historic_geo, keys);
  let valid = transitions.len() > 0;
  web::Json(json!({ "valid": valid, "date": current_dt, "geo": current_geo, "historicDate": historic_dt, "historicGeo": historic_geo, "transitions": transitions }))
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
  let num_items = vec![KeyNumValue::new("age", 98.8), KeyNumValue::new("height", 177.2)];
  web::Json( json!({ "message": "Welcome to Astro API", "time": DateInfo::now(), "extra": key_num_values_to_map(num_items) }))
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
    let port = args.port as u32;
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
          .service(body_transposed_transitions)
          .service(pheno_data)
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
  .bind(("127.0.0.1", 8087))?
  .run()
  .await
}
