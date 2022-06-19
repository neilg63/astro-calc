mod lib;
mod extensions;

extern crate serde_derive;
extern crate serde_json;
extern crate libswe_sys;  
extern crate ordered_float;
extern crate chrono;

//use libswe_sys::sweconst::{Bodies, Calandar, HouseSystem};
use libswe_sys::sweconst::{
    Bodies, Calandar, OptionalFlag,
};
use libswe_sys::swerust::{
    handler_swe02::*, handler_swe03::*, handler_swe08::*, handler_swe14::*,
};
use serde::{Serialize, Deserialize};
use serde_json::*;
use clap::Parser;
use lib::{transposed_transitions::*, transitions::*};
use lib::{core::*, models::{geo_pos::*, graha_pos::*, houses::*, date_info::*}, utils::{validators::*}};
use extensions::swe::{set_sid_mode};
use std::sync::Mutex;
use actix_web::{get, App, HttpServer, Responder, HttpRequest, web::{self, Data}};
use std::path::Path;

const SWEPH_PATH_DEFAULT: &str = "/Users/neil/apps/findingyou/findingyou-api/src/astrologic/ephe";
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
  /* let parts: Vec<f64> = loc.split(",").into_iter().map(|p| p.parse::<f64>().unwrap()).collect();
  let geo = GeoPos::new(parts[0], parts[1], 0f64); */
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
          .service(body_transposed_transitions)
          .route("/{sec1}", web::get().to(route_not_found))
          .route("/{sec1}/{sec2}", web::get().to(route_not_found))
          .route("/{sec1}/{sec2}/{sec3}", web::get().to(route_not_found))
      } else {
        App::new()
        .app_data(Data::clone(&data))
          .route("/", web::get().to(welcome_not_configured))
          .route("/{sec1}", web::get().to(welcome_not_configured))
          .route("/{sec1}/{sec2}", web::get().to(welcome_not_configured))
          .route("/{sec1}/{sec2}/{sec3}", web::get().to(welcome_not_configured))
      }
  })
  .bind(("127.0.0.1", 8087))?
  .run()
  .await
}
/* 
fn dev_test() {
  
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

  let transit_set = calc_transition_set(julian_day_ut, Bodies::Sun,  56.1f64, -3.4f64);
  let dt = julian_day_to_iso_datetime(julian_day_ut);
  println!("transits on {}: {:?}", dt, transit_set);


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


  let eq_result = calc_body_eq_jd(julian_day_ut, "me", false);

  println!("eq result: {:?}", eq_result);

  let dual_result = calc_body_dual_jd(julian_day_ut, "me", false);

  println!("dual result: {:?}", dual_result);

  let dual_result = get_bodies_dual_geo(julian_day_ut, vec!["su", "mo", "me", "ve", "ma", "ju", "sa", "ur", "ne", "pl"]);

  println!("dual result: {:?}", dual_result);

  let dual_result_topo = get_bodies_dual_topo(julian_day_ut, vec!["su", "mo", "me", "ve", "ma", "ju", "sa", "ur", "ne", "pl"], geo);

  println!("dual result topo: {:?}", dual_result_topo);

  let now_jd = current_jd();
  let transition_sets = get_transition_sets(now_jd, vec!["su", "mo", "ve", "ma"], geo);

  println!("transition sets: {:?}", serde_json::to_string(&transition_sets).unwrap() );

  let ma_positions = calc_body_positions_jd_geo(julian_day_ut, "ma", 100, 2f64);
  println!("mars positions: {:?}", ma_positions);

  let future = datetime_to_julian_day("2023-09-25");
  let bodies_positions = calc_bodies_positions_jd_geo(future, vec!["su", "mo", "me", "ve", "ma", "ju", "sa"], 50, 4f64);
  println!("body positions at {}: {:?}", future, bodies_positions);

  let past_jd = datetime_to_julian_day("1972-03-21");
  let transposed_positions = calc_transposed_graha_transitions_from_source_refs_geo(julian_day_ut, geo, past_jd, vec!["su", "mo", "me", "ve", "ma", "ju", "sa"]);
  println!("body positions at {}: {:?}", past_jd, transposed_positions);
} */
