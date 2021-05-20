use core_lib::model::document::{Document, DocumentPart, EncryptedDocument};
use core_lib::errors::*;
use reqwest::{Client, StatusCode};
use reqwest::header::{CONTENT_TYPE, HeaderValue};
use core_lib::constants::ROCKET_DOC_TYPE_API;

/// Update this token to run tests successfully that require authentication
pub const TOKEN: &'static str = "eyJ0eXAiOiJKV1QiLCJraWQiOiJkZWZhdWx0IiwiYWxnIjoiUlMyNTYifQ.eyJzY29wZXMiOlsiaWRzYzpJRFNfQ09OTkVDVE9SX0FUVFJJQlVURVNfQUxMIl0sImF1ZCI6Imlkc2M6SURTX0NPTk5FQ1RPUlNfQUxMIiwiaXNzIjoiaHR0cHM6Ly9kYXBzLmFpc2VjLmZyYXVuaG9mZXIuZGUiLCJuYmYiOjE2MTUzNzAwNjIsImlhdCI6MTYxNTM3MDA2MiwianRpIjoiTVRVeU9EVXdPRFU1TkRVMk1UY3hNRE01TnpjPSIsImV4cCI6MTYxNTM3MzY2Miwic2VjdXJpdHlQcm9maWxlIjoiaWRzYzpUUlVTVEVEX0NPTk5FQ1RPUl9TRUNVUklUWV9QUk9GSUxFIiwicmVmZXJyaW5nQ29ubmVjdG9yIjoiaHR0cDovL2NsZWFyaW5naG91c2V0ZXN0Y29ubmVjdG9yMS5kZW1vIiwiQHR5cGUiOiJpZHM6RGF0UGF5bG9hZCIsIkBjb250ZXh0IjoiaHR0cHM6Ly93M2lkLm9yZy9pZHNhL2NvbnRleHRzL2NvbnRleHQuanNvbmxkIiwidHJhbnNwb3J0Q2VydHNTaGEyNTYiOiIxZDRlYWNkMTQ2ZTg0MmU3YjllNjdkY2EyMWVjZjk5ZTk4NDliNmY0ZWJlYzlhYmQ4ODE2NzRmOTg2M2U3Y2VkIiwic3ViIjoiQjA6MDI6NDk6MjE6NEQ6QTU6N0M6Nzc6QTg6N0Q6MjM6RDc6MzM6RkQ6NjE6NUQ6OEU6QTU6NTY6QTc6a2V5aWQ6Q0I6OEM6Qzc6QjY6ODU6Nzk6QTg6MjM6QTY6Q0I6MTU6QUI6MTc6NTA6MkY6RTY6NjU6NDM6NUQ6RTgifQ.oIeD6VcrIHUEJ-DUZTAs5RIknQftT-ELaNMMsMbOXTRqYX99lCVNpoSa9YnXSJ9oQ6Zm3dhz_An-dzern4e3yG-0blvBuNWQmy65I7r1k7vTAWb8TETzEPSnpnIeRBMZoL8yEHSk3nzlLo2xBtGnOYiIpTYdIfDO-v0fxBtRT_8CwxlzF2xOSpojjZ-2qj-y4bglbJecDKQtVHm1opDuTc7BfaTIEmUG_2b3rMDLsE1QcpJXWzYEBUkl3-CtMeggf84xgsJNs13_86SuhnzdHvMKdUdAQcxQkEGLMRtCJ-dOtqjjdMNXj-cU5Vj5V7TwRfuO3gU3GczHEqXIkHBupg";

mod blockchain_api_client;
mod document_api_client;
mod keyring_api_client;
mod daps_api_client;
mod token_validation;
mod database_client;

fn create_test_document(pid: &String, dt_id: &String) -> Document{
    let p1 = DocumentPart::new(String::from("name"), Some(String::from("This is document part name.")));
    let p2 = DocumentPart::new(String::from("message"), Some(String::from("This is document part message.")));
    let p3 = DocumentPart::new(String::from("connector"), Some(String::from("This is document part connector.")));
    let pts = vec!(p1, p2, p3);
    let d = Document::new(pid.clone(), dt_id.clone(),pts);
    d
}

fn create_test_enc_document(id: &String, pid: &String, dt_id: &String) -> EncryptedDocument{
    let mut cts = vec!();
    let hash = String::from("yCRvbwBJcfA5xMC85DbcjzV+7x7Y0K2ohpGeQtj15EJGS27qrxsRl8ly+lutEXe1NQDBLYUnFQixNxwb7pEwYQ==");
    cts.push(String::from("1::4EBC3F1C2B8CB16C52E41424502FD112015D9C25919C2401514B5DD5B4233B65593CF0A4"));
    cts.push(String::from("2::FE2195305E95B9F931660CBA20B4707A1D92123022371CEDD2E70A538A8771EE7540D9F34845BBAEECEC"));
    let key_ct = String::from("very secure key ct");
    EncryptedDocument::new(id.clone(), pid.clone(), dt_id.clone(), hash, key_ct, cts)
}

fn create_dt_json(dt_id: &String, pid: &String) -> String{
    let begin_dt = r#"{"id":""#;
    let begin_pid = r#"","pid":""#;
    let rest = r#"","parts":[{"name":"name"},{"name":"message"},{"name":"connector"}]}"#;

    let mut json = String::from(begin_dt);
    json.push_str(dt_id);
    json.push_str(begin_pid);
    json.push_str(pid);
    json.push_str(rest);
    return json
}

fn insert_test_doc_type_into_keyring(token: &String, pid: &String, dt_id: &String) -> Result<bool>{
    let client = Client::new();
    let dt_url = format!("http://localhost:8002{}", ROCKET_DOC_TYPE_API);

    let json_data = create_dt_json(dt_id, pid);

    println!("json_data: {}", json_data);

    println!("calling {}", &dt_url);
    let mut response = client
        .post(dt_url.as_str())
        .header(CONTENT_TYPE, HeaderValue::from_static("application/json"))
        .bearer_auth(token)
        .body(json_data).send()?;

    println!("Status Code: {}", &response.status());
    match response.status(){
        StatusCode::CREATED => {
            println!("Response: {}", response.text()?);
            Ok(true)
        },
        _ => {
            panic!("Couldn't prepare doc type for test");
        }
    }
}

fn delete_test_doc_type_from_keyring(token: &String, pid: &String, dt_id: &String) -> Result<bool>{
    let client = Client::new();
    let dt_url = format!("http://localhost:8002{}/{}/{}", ROCKET_DOC_TYPE_API, pid, dt_id);

    println!("calling {}", &dt_url);
    let mut response = client
        .delete(dt_url.as_str())
        .header(CONTENT_TYPE, HeaderValue::from_static("application/json"))
        .bearer_auth(token)
        .send()?;

    println!("Status Code: {}", &response.status());
    match response.status(){
        StatusCode::NO_CONTENT => {
            println!("Response: {}", response.text()?);
            Ok(true)
        },
        _ => {
            println!("Couldn't delete document type");
            Ok(false)
        }
    }
}