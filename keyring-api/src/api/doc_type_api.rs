use rocket_contrib::json::Json;
use rocket::State;

use core_lib::api::ApiResponse;
use core_lib::constants::{ROCKET_DOC_TYPE_API, DEFAULT_PROCESS_ID};
use crate::db::KeyStore;
use crate::model::doc_type::DocumentType;

#[options("/<_id>")]
fn preflight(_id: Option<String>) -> ApiResponse { ApiResponse::PreFlight(()) }

#[options("/")]
fn preflight_main() -> ApiResponse { ApiResponse::PreFlight(()) }

#[post("/", format = "json", data = "<doc_type>")]
fn create_doc_type(db: State<KeyStore>, doc_type: Json<DocumentType>) -> ApiResponse {
    let doc_type: DocumentType = doc_type.into_inner();
    debug!("adding doctype: {:?}", &doc_type);
    match db.exists_document_type(&doc_type.pid, &doc_type.id){
        Ok(true) => ApiResponse::BadRequest(String::from("doctype already exists!")),
        Ok(false) => {
            match db.add_document_type(doc_type.clone()){
                Ok(()) => ApiResponse::SuccessCreate(json!(doc_type)),
                Err(e) => {
                    error!("Error while adding doctype: {:?}", e);
                    return ApiResponse::InternalError(String::from("Error while adding document type!"))
                }
            }
        },
        Err(e) => {
            error!("Error while adding document type: {:?}", e);
            return ApiResponse::InternalError(String::from("Error while checking database!"))
        }
    }
}

#[post("/<id>", format = "json", data = "<doc_type>")]
fn update_doc_type(db: State<KeyStore>, id: String, doc_type: Json<DocumentType>) -> ApiResponse {
    let doc_type: DocumentType = doc_type.into_inner();
    match db.exists_document_type(&doc_type.pid, &doc_type.id){
        Ok(true) => ApiResponse::BadRequest(String::from("Doctype already exists!")),
        Ok(false) => {
            match db.update_document_type(doc_type, &id){
                Ok(id) => ApiResponse::SuccessOk(json!(id)),
                Err(e) => {
                    error!("Error while adding doctype: {:?}", e);
                    return ApiResponse::InternalError(String::from("Error while storing document type!"))
                }
            }
        },
        Err(e) => {
            error!("Error while adding document type: {:?}", e);
            return ApiResponse::InternalError(String::from("Error while checking database!"))
        }
    }
}

#[delete("/<id>", format = "json")]
fn delete_default_doc_type(db: State<KeyStore>, id: String) -> ApiResponse{
   delete_doc_type(db, id, DEFAULT_PROCESS_ID.to_string())
}

#[delete("/<pid>/<id>", format = "json")]
fn delete_doc_type(db: State<KeyStore>, id: String, pid: String) -> ApiResponse{
    match db.delete_document_type(&id, &pid){
        Ok(true) => ApiResponse::SuccessNoContent(String::from("Document type deleted!")),
        Ok(false) => ApiResponse::NotFound(String::from("Document type does not exist!")),
        Err(e) => {
            error!("Error while deleting doctype: {:?}", e);
            ApiResponse::InternalError(format!("Error while deleting document type with id {}!", id))
        }
    }
}

#[get("/<id>", format = "json")]
fn get_default_doc_type(db: State<KeyStore>, id: String) -> ApiResponse {
    get_doc_type(db, id, DEFAULT_PROCESS_ID.to_string())
}

#[get("/<pid>/<id>", format = "json")]
fn get_doc_type(db: State<KeyStore>, id: String, pid: String) -> ApiResponse {
    match db.get_document_type(&id){
        //TODO: would like to send "{}" instead of "null" when dt is not found
        Ok(dt) => ApiResponse::SuccessOk(json!(dt)),
        Err(e) => {
            error!("Error while retrieving doctype: {:?}", e);
            ApiResponse::InternalError(format!("Error while retrieving document type with id {} and pid {}!", id, pid))
        }
    }
}

#[get("/", format = "json")]
fn get_doc_types(db: State<KeyStore>) -> ApiResponse {
    match db.get_document_types() {
        //TODO: would like to send "{}" instead of "null" when dt is not found
        Ok(dt) => ApiResponse::SuccessOk(json!(dt)),
        Err(e) => {
            error!("Error while retrieving default doctypes: {:?}", e);
            ApiResponse::InternalError(format!("Error while retrieving all document types"))
        }
    }
}

pub fn mount(rocket: rocket::Rocket) -> rocket::Rocket {
    rocket
        .mount(ROCKET_DOC_TYPE_API, routes![preflight, preflight_main, create_doc_type, update_doc_type,
        delete_default_doc_type, delete_doc_type,
        get_default_doc_type, get_doc_type , get_doc_types])
}