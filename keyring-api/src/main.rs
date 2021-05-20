#![feature(proc_macro_hygiene, decl_macro)]
extern crate chrono;
#[macro_use] extern crate error_chain;
extern crate fern;
#[macro_use] extern crate log;
#[macro_use] extern crate rocket;
#[macro_use] extern crate rocket_contrib;
#[macro_use] extern crate serde_derive;

use std::io::prelude::*;
use core_lib::{
    api::client::daps_api::DapsApiClient,
    constants::{CONFIG_FILE, INIT_DB, DAPS_API_URL, FILE_DEFAULT_DOC_TYPE},
    db::DataStoreApi,
    util,
    errors::Result,
};
use crate::db::KeyStore;
use crate::db::crypto::MasterKey;
use crate::model::doc_type::DocumentType;

mod api;
pub mod db;
mod crypto;
mod model;
#[cfg(test)] mod tests;

fn main() {

    if let Err(ref e) = launch_rocket() {
        let stderr = &mut ::std::io::stderr();
        let err_msg = "Error writing to stderr";

        writeln!(stderr, "error: {}", e).expect(err_msg);

        for e in e.iter().skip(1) {
            writeln!(stderr, "caused by: {}", e).expect(err_msg);
        }

        // The backtrace is not always generated. Try to run this example
        // with `RUST_BACKTRACE=1`.
        if let Some(backtrace) = e.backtrace() {
            writeln!(stderr, "backtrace: {:?}", backtrace).expect(err_msg);
        }

        ::std::process::exit(1);
    }
}

fn launch_rocket() -> Result<()>{

    // setup logging
    util::setup_logger()?;

    // read yaml config file
    let config = util::load_config(CONFIG_FILE);

    // init database using config.yml
    let db: KeyStore = util::configure_db(&config)?;
    // default value true
    if config[0][INIT_DB].as_bool().unwrap_or(true){
        debug!("initializing database ....");
        db.clean_db()?;
        db.create_indexes()?;
        db.store_master_key(MasterKey::new_random())?;
        let dt: DocumentType = serde_json::from_str(&util::read_file(FILE_DEFAULT_DOC_TYPE)?)?;
        db.add_document_type(dt)?;
    }

    let daps_api: DapsApiClient = util::configure_api(DAPS_API_URL, &config)?;
    let mut rocket = rocket::ignite()
        // configure db and manage it with rocket
        .manage(daps_api)
        .manage(db);
    rocket = api::key_api::mount(rocket);
    rocket = api::doc_type_api::mount(rocket);
    rocket.launch();

    Ok(())
}
