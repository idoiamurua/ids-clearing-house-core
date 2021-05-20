extern crate biscuit;
#[macro_use] extern crate bson;
extern crate chrono;
extern crate fern;
#[macro_use] extern crate log;
extern crate mongodb;
#[macro_use] extern crate rocket;
#[macro_use] extern crate serde_derive;
extern crate yaml_rust;

#[macro_use] extern crate error_chain;
pub mod errors {
    // Create the Error, ErrorKind, ResultExt, and Result types
    error_chain!{
        foreign_links {
            Io(::std::io::Error) #[cfg(unix)];
            SetLogger(log::SetLoggerError);
            ParseLogLevel(log::ParseLevelError);
            Mongodb(mongodb::Error);
            MongoOid(mongodb::oid::Error);
            MongoEncode(mongodb::EncoderError);
            MongoDecode(mongodb::DecoderError);
            Reqwest(reqwest::Error);
            SerdeJson(serde_json::error::Error);
            Uft8Error(std::string::FromUtf8Error);
            BiscuitError(biscuit::errors::Error);
        }
    }
}

pub mod api;
pub mod constants;
pub mod db;
pub mod model;
pub mod util;
