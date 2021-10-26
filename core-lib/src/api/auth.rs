use biscuit::{
    CompactJson,
    jwa::SignatureAlgorithm,
    jwk::JWKSet,
    JWT,
    CompactPart,
    Empty,
    ClaimsSet,
    ValidationOptions,
};
use rocket::outcome::IntoOutcome;
use rocket::request::{self, Request, FromRequest};
use serde::{Deserialize, Serialize};
use serde::de::DeserializeOwned;
use std::fmt::Debug;

use crate::{
    constants::{
        DAPS_AUTHHEADER,
        DAPS_AUTHBEARER,
    },
    errors::*,
    api::client::daps_api::DapsApiClient,
};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct ApiKey<T, H> {
    pub token: JWT<T, H>,
    pub raw: String,
}

impl<T, H> ApiKey<T, H>
    where
        T: CompactPart + Debug + Clone,
        H: Serialize + DeserializeOwned + Clone,
        ClaimsSet<T>: CompactPart {
    pub fn new(token: JWT<T, H>, raw: String) -> ApiKey<T, H> {
        ApiKey::<T, H> {
            token,
            raw,
        }
    }
    pub fn raw(&self) -> String {
        self.raw.clone()
    }
    pub fn issuer(&self) -> Option<String> {
        self
            .token
            .clone()
            .payload()
            .unwrap()
            .registered
            .issuer
            .clone()
    }
    pub fn sub(&self) -> Option<String> {
        self
            .token
            .clone()
            .payload()
            .unwrap()
            .registered
            .subject
            .clone()
    }
    pub fn claims(&self) -> ClaimsSet<T> {
        self
            .token
            .clone()
            .unwrap_decoded()
            .1
    }
}

#[rocket::async_trait]
impl<'r, T: DeserializeOwned + CompactJson + Debug + Clone, H: Serialize + DeserializeOwned + Clone> FromRequest<'r> for ApiKey<T, H> {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> request::Outcome<Self, ()> {
        request.rocket().state::<DapsApiClient>()
            .map(|daps_client| {
                match daps_client.get_jwks() {
                    Ok(jwks) => {
                        let auth_header_value: Vec<_> = request
                            .headers()
                            .get(&DAPS_AUTHHEADER)
                            .collect();
                        debug!("Auth Header: {:?}", &auth_header_value);
                        if auth_header_value.len() != 1 {
                            return None;
                        }
                        let mut iter = auth_header_value[0].split_ascii_whitespace();
                        match iter.next() {
                            Some(DAPS_AUTHBEARER) => {
                                let token = iter.next().unwrap_or("");
                                debug!("bearer token {:?}", &token);
                                match validate_token(token, jwks, Some(daps_client.algorithm)) {
                                    Ok(token) => {
                                        debug!("valid token!");
                                        Some(token)
                                    }
                                    Err(_e) => {
                                        debug!("invalid token {:?}, {:?}", &token, _e);
                                        None
                                    }
                                }
                            }
                            _ => None
                        }
                    }
                    Err(_e) => None
                }
            })
            .expect("DAPS client not initialized")
            .or_forward(())
    }
}


pub fn validate_token<T: Serialize + for<'de> Deserialize<'de> + CompactJson + Debug + Clone, H: Serialize + for<'de> Deserialize<'de> + Clone>(token: &str, jwks: JWKSet<Empty>, expected_algorithm: Option<SignatureAlgorithm>) -> Result<ApiKey<T, H>> {
    match JWT::new_encoded(token)
        .decode_with_jwks::<Empty>(&jwks, expected_algorithm) {
        Ok(decoded_token) => {
            match decoded_token.validate(ValidationOptions::default()) {
                Ok(()) => Ok(ApiKey::new(decoded_token, token.to_string())),
                Err(e) => Err(Error::from(e))
            }
        }
        Err(e) => Err(Error::from(e))
    }
}
