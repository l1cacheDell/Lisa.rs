use dotenvy::dotenv;
use rig::providers::openai::Client;

pub mod db_schemas;
pub mod agent_impl;
pub mod web_model;

use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use actix_web::middleware::Logger;
use env_logger::Env;

#[get("/api/ping")]
async fn ping() -> impl Responder {
    HttpResponse::Ok().body("You should use Rust.")
}

#[post("/api/chat")]
async fn chat(json: web::Json<web_model::ChatRequest>) -> actix_web::Result<impl Responder> {
    let user_id = &json.user_id;
    let prompt = &json.content;

    let response = web_model::ChatResponse {
        status: "success".to_string(),
        agent_response: "I don't have a response for that yet.".to_string(),
    };
    Ok(web::Json(response))
}

#[post("/api/store_drift_bottle")]
async fn store_drift_bottle(json: web::Json<web_model::StoreDriftBottleRequest>) -> actix_web::Result<impl Responder> {
    let wallet = &json.wallet;
    let title = &json.title;
    let drift_bottle_content = &json.content;

    let mut response = web_model::GeneralReponse {
        status: "success".to_string()
    };

    // store this drift bottle in DB
    // currently we will implement the basic connection method, no ConnPool implemented.

    db_schemas::store_drift_vec(&wallet, &title, &drift_bottle_content).await.unwrap_or_else(|e| {
        println!("Error: {}", e);
        response = web_model::GeneralReponse {
            status: format!("Error: {}", e)
        };
    });
    Ok(web::Json(response))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // load environment variables from.env file
    dotenv().ok();
    let api_key: String = std::env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY not set");
    let base_url: String = std::env::var("BASE_URL").expect("BASE_URL not set");
    let model_name: String = std::env::var("MODEL_NAME").expect("MODEL_NAME not set");

    let prompt = "What do you think of Trump?";

    const IPADDRESS: &str = "localhost";
    const PORT: u16 = 8080;
    println!("Server will be listening on http://{}:{}", IPADDRESS, PORT);

    // let response_from_qwen = create_completion(&api_key, &base_url, &model_name, &prompt).await;
    // println!("{model_name}: {response_from_qwen}");

    let openai_client = Client::from_url(&api_key, &base_url);

    let embedding_model_name = "BAAI/bge-large-en-v1.5";

    env_logger::init_from_env(Env::default().default_filter_or("info"));

    HttpServer::new(|| {
        App::new()
            .service(ping)
            .service(chat)
            .service(store_drift_bottle)
            .wrap(Logger::default())
            .wrap(Logger::new("%a"))
    })
    .workers(2)
    .bind((IPADDRESS, PORT))?
    .run()
    .await

}