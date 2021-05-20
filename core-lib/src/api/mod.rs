use rocket_contrib::json::JsonValue;
use std::string::ToString;

pub mod auth;
pub mod claims;
pub mod client;

pub trait ApiClient{
    fn new(url: &str) -> Self;
}

#[derive(Responder, Debug)]
pub enum ApiResponse {
    #[response(status = 200)]
    PreFlight(()),
    #[response(status = 400, content_type = "json")]
    BadRequest(String),
    #[response(status = 201, content_type = "json")]
    SuccessCreate(JsonValue),
    #[response(status = 200, content_type = "json")]
    SuccessOk(JsonValue),
    #[response(status = 204, content_type = "json")]
    SuccessNoContent(String),
    #[response(status = 401, content_type = "json")]
    Unauthorized(String),
    #[response(status = 404, content_type = "json")]
    NotFound(String),
    #[response(status = 500, content_type = "json")]
    InternalError(String),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BlockchainMessage {
    pub cid: String,
    pub id: String,
    pub hash: String,
}

impl BlockchainMessage {
    pub fn new(id: String, cid: String, hash: String) -> BlockchainMessage {
        BlockchainMessage { id, cid, hash }
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct HashMessage{
    pub success: String,
    pub message: String,
    pub doc_id: String,
    pub hash: String,
}

impl HashMessage{
    pub fn new(success: &str, message: &str, doc_id: &str, hash: &str) -> HashMessage{
        HashMessage{
            success: success.to_string(),
            message: message.to_string(),
            doc_id: doc_id.to_string(),
            hash: hash.to_string()
        }
    }
}
