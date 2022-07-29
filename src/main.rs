mod lib;
mod extensions;
mod constants;
mod query_params;
mod post_params;
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
use lib::{models::{date_info::*}};
use extensions::swe::{set_sid_mode};
use actix_web::{App, HttpServer, Responder, web::{self, Json}};
use std::path::Path;
use constants::*;
use help::*;
use routes::{chart_data::*, transitions::*, planet_stations::*, datetime::*, progress_synastry::*};

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

async fn date_now() -> impl Responder {
  Json(json!(DateInfo::now()))
}

async fn welcome() -> impl Responder {
  Json(json!({ "message": "Welcome to Astro API", "time": DateInfo::now(), "routes": endpoint_help(), "ephemerisPath": get_ephemeris_path() }))
}

async fn welcome_not_configured() -> impl Responder {
  Json( json!({ "valid": false, "message": "Welcome to Astro API", "error": "Incorrect ephemeris path", "time": DateInfo::now(), "ephemerisPath": get_ephemeris_path() }))
}

async fn route_not_found() -> impl Responder {
  Json( json!({ "valid": false, "error": "route not found" }))
}

fn get_ephemeris_path() -> String {
  let args = Args::parse();
  args.ephemeris
}

pub fn reset_ephemeris_path() {
  let micro_interval = time::Duration::from_millis(10);
  let ep = get_ephemeris_path();
  set_ephe_path(ep.as_str());
  thread::sleep(micro_interval);
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

    HttpServer::new(move || {
      if has_path {
        App::new()
          .route("/", web::get().to(welcome))
          .route("/jd", web::get().to(date_now))
          .service(date_info)
          .service(date_info_geo)
          .service(test_geo_start)
          .service(bodies_progress)
          .service(body_positions)
          .service(chart_data_flexi)
          .service(progress_synastry_items)
          .service(list_sun_transitions)
          .service(pheno_data)
          .service(list_transitions)
          .service(test_transitions)
          .service(test_mcs)
          .service(body_transposed_transitions_range)
          .service(planet_stations_progress)
          .service(body_transposed_transitions_from_chart)
          .route("/{sec1}", web::get().to(route_not_found))
          .route("/{sec1}/{sec2}", web::get().to(route_not_found))
          .route("/{sec1}/{sec2}/{sec3}", web::get().to(route_not_found))
          .route("/{sec1}/{sec2}/{sec3}/{sec4}", web::get().to(route_not_found))
      } else {
        App::new()
          .route("/", web::get().to(welcome_not_configured))
          .route("/{sec1}", web::get().to(welcome_not_configured))
          .route("/{sec1}/{sec2}", web::get().to(welcome_not_configured))
          .route("/{sec1}/{sec2}/{sec3}", web::get().to(welcome_not_configured))
          .route("/{sec1}/{sec2}/{sec3}/{sec4}", web::get().to(route_not_found))
      }
  })
  .bind(("127.0.0.1", port))?
  .run()
  .await
}
