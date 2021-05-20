use reqwest::Client;
use reqwest::StatusCode;
use reqwest::header::{HeaderValue, CONTENT_TYPE};
use serde_json;
use crate::api::{ApiClient, HashMessage};
use crate::constants::{ROCKET_DOC_API};
use crate::errors::*;
use crate::model::document::Document;
use crate::util::url_encode;

#[derive(Clone)]
pub struct DocumentApiClient {
    uri: String,
}

impl ApiClient for DocumentApiClient {
    fn new(uri: &str) -> DocumentApiClient {
        let uri = String::from(uri);
        DocumentApiClient {
            uri: uri,
        }
    }
}

impl DocumentApiClient{
    pub fn get_document(&self, token: &String, pid: &String, id: &String) -> Result<Document>{
        let document_url = format!("{}{}/{}/{}", self.uri, ROCKET_DOC_API, url_encode(pid), url_encode(id));
        let client = Client::new();

        debug!("calling {}", &document_url);
        let mut response = client
            .get(document_url.as_str())
            .header(CONTENT_TYPE, HeaderValue::from_static("application/json"))
            .bearer_auth(token)
            .send()?;

        debug!("Status Code: {}", &response.status());
        let doc: Document = response.json()?;
        Ok(doc)
    }

    pub fn get_document_with_integrity_check(&self, token: &String, pid: &String, id: &String, hash: &String) -> Result<Document>{
        let document_url = format!("{}{}/{}/{}", self.uri, ROCKET_DOC_API, url_encode(pid), url_encode(id));
        let client = Client::new();

        debug!("calling {}", &document_url);
        let mut response = client
            .get(document_url.as_str())
            .header(CONTENT_TYPE, HeaderValue::from_static("application/json"))
            .query(&[("hash", hash.as_str())])
            .bearer_auth(token)
            .send()?;

        debug!("Status Code: {}", &response.status());
        let doc: Document = response.json()?;
        Ok(doc)
    }

    pub fn get_documents_for_pid(&self, token: &String, pid: &String) -> Result<Vec<Document>>{
        let document_url = format!("{}{}/{}", self.uri, ROCKET_DOC_API, url_encode(pid));
        let client = Client::new();

        debug!("calling {}", &document_url);
        let mut response = client
            .get(document_url.as_str())
            .header(CONTENT_TYPE, HeaderValue::from_static("application/json"))
            .bearer_auth(token)
            .send()?;

        debug!("Status Code: {}", &response.status());
        let docs: Vec<Document> = response.json()?;
        Ok(docs)
    }

    pub fn create_document(&self, token: &String, doc: &Document) -> Result<HashMessage> {
        let document_url = format!("{}{}", self.uri, ROCKET_DOC_API);
        let client = Client::new();

        let json_data = serde_json::to_string(doc)?;
        debug!("calling {}", &document_url);
        let mut response = client
            .post(document_url.as_str())
            .header(CONTENT_TYPE, HeaderValue::from_static("application/json"))
            .bearer_auth(token)
            .body(json_data).send()?;

        debug!("Status Code: {}", &response.status());
        match &response.status(){
            &StatusCode::CREATED => {
                let hash_message = response.json()?;
                println!("Payload: {:?}", hash_message);
                Ok(hash_message)
            },
            _ => bail!("Error while calling create_document(): status {} content {:?}", response.status(), response.text())
        }

    }

    pub fn delete_document(&self, token: &String, pid: &String, id: &String) -> Result<bool>{
        let document_url = format!("{}{}/{}/{}", self.uri, ROCKET_DOC_API, url_encode(pid), url_encode(id));
        let client = Client::new();

        debug!("calling {}", &document_url);
        let response = client
            .delete(document_url.as_str())
            .header(CONTENT_TYPE, HeaderValue::from_static("application/json"))
            .bearer_auth(token)
            .send()?;

        debug!("Status Code: {}", &response.status());
        Ok(response.status().is_success())
    }
 }