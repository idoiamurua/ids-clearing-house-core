use biscuit::Empty;
use rocket_contrib::json::Json;
use rocket::State;
use core_lib::{
    api::{
        ApiResponse,
        auth::ApiKey,
        claims::IdsClaims,
        client::keyring_api::KeyringApiClient,
        HashMessage
    },
    constants::ROCKET_DOC_API,
    db::DataStore,
    model::{
        document::Document
    }
};

#[options("/<_id>")]
fn preflight(_id: Option<String>) -> ApiResponse { ApiResponse::PreFlight(()) }

#[options("/")]
fn preflight_main() -> ApiResponse { ApiResponse::PreFlight(()) }

#[post("/", format = "json", data = "<document>")]
fn create_enc_document(
    api_key: ApiKey<IdsClaims, Empty>,
    db: State<DataStore>,
    key_api: State<KeyringApiClient>,
    document: Json<Document>
) -> ApiResponse {
    debug!("user '{:?}' with claims {:?}", api_key.sub(), api_key.claims());
    let doc: Document = document.into_inner();
    trace!("requested document is: '{:#?}'", json!(doc));

    // check if doc id already exists
    match db.exists_document(&doc.id) {
        Ok(true) => {
            warn!("Document exists already!");
            ApiResponse::BadRequest(String::from("Document exists already!"))
        },
        _ => {
            debug!("Document does not exists!");

            //TODO: get keys to encrypt document
            debug!("getting keys");
            let keys;
            match key_api.generate_keys(&api_key.raw(), &doc.pid, &doc.dt_id) {
                Ok(key_map) => {
                    keys = key_map;
                    debug!("got keys");
                },
                Err(e) => {
                    error!("Error while retrieving keys: {:?}", e);
                    return ApiResponse::InternalError(String::from("Error while retrieving keys!"))
                },
            };

            debug!("start encryption");
            let enc_doc;
            match doc.encrypt(keys) {
                Ok(ct) => {
                    debug!("got ct");
                    enc_doc = ct
                },
                Err(e) => {
                    error!("Error while encrypting: {:?}", e);
                    return ApiResponse::InternalError(String::from("Error while encrypting!"))
                },
            };

            // prepare the success result message
            let res = HashMessage::new("true", "Document created", &enc_doc.id, enc_doc.hash.as_str());

            debug!("storing document ....");
            // store document
            //TODO store encrypted keys
            match db.add_document(enc_doc) {
                Ok(_b) => ApiResponse::SuccessCreate(json!(res)),
                Err(e) => {
                    error!("Error while adding: {:?}", e);
                    ApiResponse::InternalError(String::from("Error while storing document!"))
                }
            }
        }
    }
}

#[delete("/<pid>/<id>", format = "json")]
fn delete_document(api_key: ApiKey<IdsClaims, Empty>, db: State<DataStore>, pid: String, id: String) -> ApiResponse {
    debug!("delete called...");
    debug!("user '{:?}' with claims {:?}", api_key.sub(), api_key.claims());
    // this is only a sanity check, i.e. we make sure id/pid pair exists
    match db.get_document(&id, &pid){
        Ok(Some(_enc_doc)) => {
            match db.delete_document(&id){
                Ok(true) => ApiResponse::SuccessNoContent(String::from("Document deleted!")),
                Ok(false) => ApiResponse::NotFound(String::from("Document does not exist!")),
                Err(e) => {
                    error!("Error while deleting document: {:?}", e);
                    ApiResponse::InternalError(format!("Error while deleting document {}!", &id))
                }
            }
        }
        _ => {
            warn!("Document '{}' with pid '{}' not found!", &id, &pid);
            ApiResponse::NotFound(String::from("Document to delete not found"))
        }
    }
}

#[get("/<pid>?<doc_type>", format = "json")]
fn get_enc_documents_for_pid(api_key: ApiKey<IdsClaims, Empty>, key_api: State<KeyringApiClient>, db: State<DataStore>, doc_type: Option<String>, pid: String) -> ApiResponse {
    debug!("trying to retrieve documents for pid '{}'", &pid);
    debug!("user '{:?}' with claims {:?}", api_key.sub(), api_key.claims());
    // either call db with type filter or without to get cts
    let cts;
    if doc_type.is_some(){
        debug!("but only of document type: '{}'", doc_type.as_ref().unwrap());
        match db.get_documents_of_dt_for_pid(doc_type.as_ref().unwrap(), &pid){
            Ok(cts_type_filter) => cts = cts_type_filter,
            Err(e) => {
                error!("Error while retrieving document: {:?}", e);
                return ApiResponse::InternalError(format!("Error while retrieving document for {}", &pid))
            }
        }
    }
    else{
        debug!("no type filter applied");
        match db.get_documents_for_pid(&pid){
            //TODO: would like to send "{}" instead of "null" when dt is not found
            Ok(cts_unfiltered) => cts = cts_unfiltered,
            Err(e) => {
                error!("Error while retrieving document: {:?}", e);
                return ApiResponse::InternalError(format!("Error while retrieving document for {}", &pid))
            }
        }
    };
    debug!("Found {} documents.", cts.len());
    // decrypt cts
    let pts : Vec<Document>= cts.iter()
            .filter_map(|ct| {
                match hex::decode(&ct.keys_ct){
                    Ok(key_ct) => {
                        match key_api.decrypt_keys(&api_key.raw(), &pid, &ct.dt_id, &key_ct){
                            Ok(key_map) => {
                                match ct.decrypt(key_map.keys, None){
                                    Ok(d) => Some(d),
                                    Err(e) => {
                                        warn!("Got empty document from decryption! {:?}", e);
                                        None
                                    }
                                }
                            }
                            Err(e) => {
                                error!("Error while retrieving keys from keyring: {:?}", e);
                                None
                            }
                        }
                    },
                    Err(e) => {
                        error!("Error while decoding key ciphertext: {:?}", e);
                        None
                    }
                }
            })
            .collect();
    ApiResponse::SuccessOk(json!(pts))
}

/// Retrieve document with id for process with pid
#[get("/<pid>/<id>?<hash>", format = "json")]
fn get_enc_document(api_key: ApiKey<IdsClaims, Empty>, key_api: State<KeyringApiClient>, db: State<DataStore>, pid: String, id: String, hash: Option<String>) -> ApiResponse {
    debug!("user '{:?}' with claims {:?}", api_key.sub(), api_key.claims());
    debug!("trying to retrieve document with id '{}' for pid '{}'", &id, &pid);
    if hash.is_some(){
        debug!("integrity check with hash: {}", hash.as_ref().unwrap());
    }

    match db.get_document(&id, &pid){
        //TODO: would like to send "{}" instead of "null" when dt is not found
        Ok(Some(ct)) => {
            match hex::decode(&ct.keys_ct){
                Ok(key_ct) => {
                    match key_api.decrypt_keys(&api_key.raw(), &pid, &ct.dt_id, &key_ct){
                        Ok(key_map) => {
                            match ct.decrypt(key_map.keys, hash){
                                Ok(d) => ApiResponse::SuccessOk(json!(d)),
                                Err(e) => {
                                    warn!("Got empty document from decryption! {:?}", e);
                                    return ApiResponse::NotFound(format!("Document {} not found!", &id))
                                }
                            }
                        }
                        Err(e) => {
                            error!("Error while retrieving keys from keyring: {:?}", e);
                            return ApiResponse::InternalError(format!("Error while retrieving keys"))
                        }
                    }

                },
                Err(e) => {
                    error!("Error while decoding ciphertext: {:?}", e);
                    return ApiResponse::InternalError(format!("Key Ciphertext corrupted"))
                }
            }
        },
        Ok(None) => {
            debug!("Nothing found in db!");
            return ApiResponse::NotFound(format!("Document {} not found!", &id))
        }
        Err(e) => {
            error!("Error while retrieving document: {:?}", e);
            return ApiResponse::InternalError(format!("Error while retrieving document {}", &id))
        }
    }
}

pub fn mount(rocket: rocket::Rocket) -> rocket::Rocket {
    rocket
        .mount(ROCKET_DOC_API, routes![preflight, preflight_main, create_enc_document, delete_document,
                                            get_enc_document, get_enc_documents_for_pid])
}