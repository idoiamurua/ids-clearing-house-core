use rocket::State;
use core_lib::{
    constants::ROCKET_KEYRING_API,
    api::{
        ApiResponse,
        auth::ApiKey,
        claims::IdsClaims
    },
};
use crate::crypto::{generate_key_map, restore_key_map};
use crate::db::KeyStore;
use biscuit::{
    Empty
};

#[get("/generate_keys/<_pid>?<dt_id>", format = "json")]
fn generate_keys(api_key: ApiKey<IdsClaims, Empty>, db:State<KeyStore>, _pid: Option<String>, dt_id: String) -> ApiResponse {
    debug!("user '{:?}' with claims {:?}", api_key.sub(), api_key.claims());
    // get master key
    match db.get_msk(){
        Ok(key) => {
            // check that doc type exists for pid
            match db.get_document_type(&dt_id){
                Ok(dt) => {
                    // generate new random key map
                    match generate_key_map(key, dt) {
                        Ok(key_map) => {
                            debug!("response: {:?}", &key_map);
                            return ApiResponse::SuccessCreate(json!(key_map));
                        },
                        Err(e) => {
                            error!("Error while generating key map: {}", e);
                            return ApiResponse::InternalError(String::from("Error while generating keys"));
                        }
                    }
                }
                Err(e) => {
                    warn!("Error while retrieving document type: {}", e);
                    return ApiResponse::NotFound(String::from("Document type not found!"));
                }
            }
        }
        Err(e) => {
            error!("Error while retrieving master key: {}", e);
            return ApiResponse::InternalError(String::from("Error while generating keys"));
        }
    }
}

#[get("/decrypt_keys/<_pid>/<keys_ct>?<dt_id>", format = "json")]
fn decrypt_keys(api_key: ApiKey<IdsClaims, Empty>, db:State<KeyStore>, keys_ct: String, _pid: Option<String>, dt_id: String) -> ApiResponse {
    debug!("user '{:?}' with claims {:?}", api_key.sub(), api_key.claims());
    debug!("ct: {}", &keys_ct);
    // get master key
    match db.get_msk(){
        Ok(key) => {
            // check that doc type exists for pid
            match db.get_document_type(&dt_id){
                Ok(dt) => {
                    // validate keys_ct input
                    let keys_ct = match hex::decode(keys_ct){
                        Ok(key) => key,
                        Err(e) => {
                            error!("Error while decoding key ciphertext: {}", e);
                            return ApiResponse::InternalError(String::from("Error while decrypting keys"));
                        }
                    };

                    match restore_key_map(key, dt, keys_ct){
                        Ok(key_map) => {
                            return ApiResponse::SuccessOk(json!(key_map));
                        },
                        Err(e) => {
                            error!("Error while generating key map: {}", e);
                            return ApiResponse::InternalError(String::from("Error while restoring keys"));
                        }
                    }
                }
                Err(e) => {
                    warn!("Error while retrieving document type: {}", e);
                    return ApiResponse::NotFound(String::from("Document type not found!"));
                }
            }
        }
        Err(e) => {
            error!("Error while retrieving master key: {}", e);
            return ApiResponse::InternalError(String::from("Error while decrypting keys"));
        }
    }
}

pub fn mount(rocket: rocket::Rocket) -> rocket::Rocket {
    rocket
        .mount(ROCKET_KEYRING_API, routes![decrypt_keys, generate_keys])
}