use dotenvy::dotenv;
use rig::providers::openai::Client;

pub mod db_schemas;
pub mod agent_impl;
pub mod web_model;

use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use actix_web::middleware::Logger;
use env_logger::Env;
use rusqlite::ffi::sqlite3_auto_extension;
use sqlite_vec::sqlite3_vec_init;

#[get("/api/ping")]
async fn ping() -> actix_web::Result<impl Responder> {
    Ok(web::Json(web_model::GeneralReponse {
        status: "pong".to_string()
    }))
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

    unsafe {
        sqlite3_auto_extension(Some(std::mem::transmute(sqlite3_vec_init as *const ())));
    }

    const IPADDRESS: &str = "localhost";
    let port: u16 = std::env::var("PORT").unwrap_or("8080".to_string()).parse::<u16>().expect("Invalid port number");
    println!("Server will be listening on http://{}:{}", IPADDRESS, port);

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
    .bind((IPADDRESS, port))?
    .run()
    .await

}