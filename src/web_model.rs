use serde::{de, Deserialize, Serialize};

// chat api
#[derive(Deserialize)]
pub struct ChatRequest {
    pub user_id: String,
    pub content: String
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