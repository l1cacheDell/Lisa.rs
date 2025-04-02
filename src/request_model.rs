use serde::{de, Deserialize, Serialize};
use crate::db_schemas::DocInfo;

// chat api
#[derive(Deserialize)]
pub struct ChatRequest {
    pub wallet: String,
    pub content: String,
}

#[derive(Serialize)]
pub struct ChatResponse {
    pub status: String,
    pub agent_response: String
}

// store drift bottle api
#[derive(Deserialize)]
pub struct StoreDriftBottleRequest {
    pub wallet: String,
    pub title: String,
    pub content: String
}

#[derive(Serialize)]
pub struct GeneralReponse {
    pub status: String
}

// retrive drift bottle api
#[derive(Deserialize)]
pub struct RetriveRequest {
    pub wallet: String,
    pub content: String,
}

#[derive(Serialize)]
pub struct RetriveResponse {
    pub status: String,
    pub retrive_results: Vec<DocInfo>
}

// grade drift bottle score api
#[derive(Deserialize)]
pub struct GradeBottleRequest {
    pub wallet: String,
    pub title: String,
    pub content: String,
    pub tx_hash: String
}

#[derive(Serialize)]
pub struct GradeBottleResponse {
    pub status: String,
    pub score: u8   // 0-100
}