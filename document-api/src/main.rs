#![feature(proc_macro_hygiene, decl_macro)]

extern crate chrono;
extern crate fern;
#[macro_use] extern crate log;
#[macro_use] extern crate rocket;
#[macro_use] extern crate rocket_contrib;
extern crate rocket_cors;


use std::io::prelude::*;
use core_lib::{
    api::{
        ApiResponse,
        client::{
            keyring_api::KeyringApiClient,
            daps_api::DapsApiClient
        },
    },
    db::{DataStore, DataStoreApi},
    constants::{
        CONFIG_FILE,
        KEYRING_API_URL,
        DAPS_API_URL,
        INIT_DB,
        ROCKET_STATISTICS},
    util,
    errors::*
};
use rocket::http::Method;
use rocket_contrib::json::JsonValue;
use rocket_cors::{
    AllowedHeaders, AllowedOrigins,
    Cors, CorsOptions
};
use rocket::State;
mod doc_api;

fn make_cors() -> Cors {
    CorsOptions {
        allowed_origins: AllowedOrigins::some_exact(&[
            "http://127.0.0.1",
            "http://127.0.0.1:4200",
            "http://127.0.0.1:8001",
            "http://localhost",
            "http://localhost:4200",
            "http://localhost:8001",
            "http://document-gui",
            "http://document-gui.local",
            "https://127.0.0.1",
            "https://127.0.0.1:4200",
            "https://127.0.0.1:8001",
            "https://localhost",
            "https://localhost:4200",
            "https://localhost:8001",
            "https://document-gui",
            "https://document-gui.local"
        ]),
        allowed_methods: vec![Method::Get, Method::Post, Method::Options, Method::Delete].into_iter().map(From::from).collect(),
        allowed_headers: AllowedHeaders::some(&[
            "Access-Control-Allow-Origin",
            "Access-Control-Allow-Methods",
            "Access-Control-Allow-Headers",
            "Accept",
            "Authorization",
            "Content-Type",
            "Origin"

        ]),
        allow_credentials: true,
        ..Default::default()
    }
        .to_cors()
        .expect("error while building CORS")
}

#[options("/statistics")]
fn preflight_statistics() -> ApiResponse {
    ApiResponse::SuccessNoContent("".to_string())
}


#[get("/statistics", format = "json")]
fn statistics(db: State<DataStore>) -> ApiResponse {
    debug!("stats called...");
    match db.statistics(){
        Ok(stats) => ApiResponse::SuccessOk(JsonValue::from(stats)),
        Err(e) => {
            error!("Error while getting statistics: {}", e);
            ApiResponse::InternalError(format!("Error while getting statistics: {}", e))
        }
    }
}

#[options("/metrics")]
fn preflight_metrics() -> ApiResponse {
    ApiResponse::SuccessNoContent("".to_string())
}

#[get("/metrics", format = "json")]
fn metrics(db: State<DataStore>) -> ApiResponse {
    debug!("metrics called...");
    match db.metrics(){
        Ok(result) => {
            let metrics = result["metrics"].as_object().unwrap().get("document").unwrap().clone();
            ApiResponse::SuccessOk(JsonValue::from(metrics))
        },
        Err(e) => {
            error!("Error while getting metrics: {}", e);
            ApiResponse::InternalError(format!("Error while getting metrics: {}", e))
        }
    }
}
fn main() {
    if let Err(ref e) = launch_rocket() {
        let stderr = &mut ::std::io::stderr();
        let errmsg = "Error writing to stderr";

        writeln!(stderr, "error: {}", e).expect(errmsg);

        for e in e.iter().skip(1) {
            writeln!(stderr, "caused by: {}", e).expect(errmsg);
        }

        // The backtrace is not always generated. Try to run this example
        // with `RUST_BACKTRACE=1`.
        if let Some(backtrace) = e.backtrace() {
            writeln!(stderr, "backtrace: {:?}", backtrace).expect(errmsg);
        }

        ::std::process::exit(1);
    }
}

fn launch_rocket() -> Result<()>{

    // setup logging
    util::setup_logger()?;

    // read yaml config file
    let config = util::load_config(CONFIG_FILE);

    let db: DataStore = util::configure_db(&config)?;
    // default value = true
    if config[0][INIT_DB].as_bool().unwrap_or(true) {
        db.clean_db()?;
        db.create_indexes()?;
    }
    db.db_exists()?;
    let key_api: KeyringApiClient = util::configure_api(KEYRING_API_URL, &config)?;
    let daps_api: DapsApiClient = util::configure_api(DAPS_API_URL, &config)?;
    let mut rocket = rocket::ignite()
        // configure db and manage it with rocket
        .manage(db)
        .manage(key_api)
        .manage(daps_api)
        .attach(make_cors());
    rocket = doc_api::mount(rocket);
    rocket.mount(ROCKET_STATISTICS, routes![statistics, preflight_metrics, metrics, preflight_statistics]).launch();

    Ok(())
}