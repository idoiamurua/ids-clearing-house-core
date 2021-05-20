use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::convert::TryFrom;
use std::str::FromStr;
use yaml_rust::yaml::Yaml;
use yaml_rust::YamlLoader;

use crate::api::ApiClient;
use crate::db::DataStoreApi;
use crate::constants::{DATABASE_URL, DATABASE_PORT, ENV_API_LOG_LEVEL};
use crate::errors::*;

// read yaml file
pub fn load_config(file: &str) -> Vec<Yaml> {
    let mut file = File::open(file).expect("Unable to open file");
    let mut contents = String::new();

    file.read_to_string(&mut contents).expect("Unable to read file");

    YamlLoader::load_from_str(&contents).unwrap()
}


// get params from yaml and configure Database
pub fn configure_db<T: DataStoreApi>(config: &Vec<Yaml>) -> Result<T>{
    let db_url;
    match &config[0][DATABASE_URL].as_str() {
        Some(url) => db_url = url.clone(),
        None => {panic!{"Database URL missing in config file!"}}
    };

    let db_port;
    match config[0][DATABASE_PORT].as_i64() {
        Some(port) => {
            db_port = u16::try_from(port).unwrap();
        },
        None => {panic!{"Database Port missing in config file!"}}
    };
    Ok(DataStoreApi::new(db_url, db_port))
}

// get params from yaml and configure Api-Client
pub fn configure_api<T: ApiClient>(url: &str, config: &Vec<Yaml>) -> Result<T>{
    let api_url;
    match &config[0][url].as_str() {
        Some(url) => api_url = url.clone(),
        None => {panic!{"Api URL missing in config file!"}}
    };
    Ok(ApiClient::new(api_url))
}

/// setup the fern logger and set log level to environment variable `ENV_API_LOG_LEVEL`
/// allowed levels: `Off`, `Error`, `Warn`, `Info`, `Debug`, `Trace`
pub fn setup_logger() -> Result<()> {
    let log_level;
    match env::var(ENV_API_LOG_LEVEL){
        Ok(l) => log_level = l.clone(),
        Err(_e) => {
            println!("Error with log level. Logging disabled");
            log_level = String::from("Off")
        }
    };

    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{}[{}][{}] {}",
                chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                record.target(),
                record.level(),
                message
            ))
        })
        .level(log::LevelFilter::from_str(&log_level.as_str())?)
        .chain(std::io::stdout())
        .chain(fern::log_file("output.log")?)
        .apply()?;
    Ok(())
}

pub fn read_file(file: &str) -> Result<String> {
    let mut f = File::open(file)?;
    let mut data = String::new();
    f.read_to_string(&mut data)?;
    drop(f);
    Ok(data)
}

pub fn url_encode(id: &str) -> String{
    utf8_percent_encode(id, NON_ALPHANUMERIC).to_string()
}