use reqwest::Client;
use serde_json;
use biscuit::{
    jwa::SignatureAlgorithm,
    jwk::JWKSet
};
use crate::api::ApiClient;
use crate::errors::*;
use crate::constants::DAPS_JWKS;
use serde::de::DeserializeOwned;

#[derive(Clone)]
pub struct DapsApiClient {
    uri: String,
    pub algorithm: SignatureAlgorithm
}

impl Default for DapsApiClient {
    fn default() -> Self {
        DapsApiClient {
            uri: "".to_string(),
            algorithm: SignatureAlgorithm::RS256
        }
    }
}

impl ApiClient for DapsApiClient {
    fn new(s: &str) -> DapsApiClient {
        DapsApiClient {
            uri: String::from(s),
            ..Default::default()
        }
    }
}

impl DapsApiClient {
    pub fn get_jwks<J: DeserializeOwned>(&self) -> Result<JWKSet<J>>{
        let pk_url = format!("{}/{}", self.uri, DAPS_JWKS);
        debug!("trying to get JWKSet from url: {}", pk_url);
        //TODO Fix daps server certificate!
        let client = Client::builder().danger_accept_invalid_certs(true).build().unwrap();
        match client.get(pk_url.as_str()).send() {
            Ok(mut resp) => {
                match serde_json::from_str(&resp.text().unwrap()) {
                    Ok(body) => {
                        Ok(body)
                    },
                    Err(e) =>{
                        error!("error while parsing answer from server: {:?}", e);
                        Err(Error::from(e))
                    }
                }
            },
            Err(e) => {
                error!("did not receive response from {}: {:?}", pk_url, e);
                Err(Error::from(e))
            },
        }
    }
}