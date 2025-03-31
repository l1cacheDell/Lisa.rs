use dotenvy::dotenv;
use rig::providers::openai::Client;
use rig::completion::Prompt;

pub mod db_schemas;
pub mod agent_impl;
pub mod request_model;
pub mod test_sqlite_vec;

use request_model::{ChatRequest, ChatResponse, GeneralReponse, RetriveRequest, RetriveResponse};
use agent_impl::{RetrivalAgent, prompt_hub, RetrivalTool};

use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use actix_web::middleware::Logger;
use env_logger::Env;
use rusqlite::ffi::sqlite3_auto_extension;
use sqlite_vec::sqlite3_vec_init;

#[get("/api/ping")]
async fn ping() -> actix_web::Result<impl Responder> {
    Ok(web::Json(GeneralReponse {
        status: "pong".to_string()
    }))
}

#[post("/api/chat")]
async fn chat(json: web::Json<ChatRequest>) -> actix_web::Result<impl Responder> {
    let wallet = &json.wallet;
    let prompt = &json.content;
    let tx_hash = &json.tx_hash;

    let mut response = ChatResponse {
        status: "success".to_string(),
        agent_response: "I don't have a response for that yet.".to_string(),
    };

    let valid_tx: bool = true;
    if !valid_tx {
        response = ChatResponse {
            status: "Fail to response".to_string(),
            agent_response: "The transaction hash was verified as invalid, check your payment".to_string(),
        };
        return Ok(web::Json(response));
    }

    let sys_prompt = prompt_hub::CHAT_AGENT_SYS_PROMPT;

    let chat_agent = RetrivalAgent::new(
        sys_prompt.to_string(), 
        Some(128), 
        Some(0.9), 
        Some(1)).await.unwrap();

    let agent_response = chat_agent.prompt(prompt.clone()).await.unwrap_or_else(|e| {
        println!("An error occured! {e}");
        "Fail to process".to_string()
    });

    response = ChatResponse {
        status: "success".to_string(),
        agent_response
    };

    Ok(web::Json(response))
}

#[post("/api/store_drift")]
async fn store_drift(json: web::Json<request_model::StoreDriftBottleRequest>) -> actix_web::Result<impl Responder> {
    let wallet = &json.wallet;
    let title = &json.title;
    let drift_bottle_content = &json.content;

    let mut response = request_model::GeneralReponse {
        status: "success".to_string()
    };

    // store this drift bottle in DB
    // currently we will implement the basic connection method, no ConnPool implemented.

    db_schemas::store_drift_vec(&wallet, &title, &drift_bottle_content).await.unwrap_or_else(|e| {
        response = request_model::GeneralReponse {
            status: format!("Error: {}", e)
        };
    });
    Ok(web::Json(response))
}

// #[get("/api/grade_drift")]
// async fn grade_drift() -> actix_web::Result<impl Responder> {

//     Ok(web::Json(response))
// }

#[get("/api/retrive_drift")]
async fn retrive_drift(json: web::Json<request_model::RetriveRequest>) -> actix_web::Result<impl Responder> {
    let wallet = &json.wallet;
    let prompt = &json.content;
    let tx_hash = &json.tx_hash;

    let mut response = RetriveResponse {
        status: "success".to_string(),
        agent_response: "I don't have a response for that yet.".to_string(),
    };

    let valid_tx: bool = true;
    if !valid_tx {
        response = RetriveResponse {
            status: "Fail to response".to_string(),
            agent_response: "The transaction hash was verified as invalid, check your payment".to_string(),
        };
        return Ok(web::Json(response));
    }

    let sys_prompt = prompt_hub::RETRIVAL_AGENT_SYS_PROMPT;

    let retrival_agent_builder = RetrivalAgent::new_builder(
        sys_prompt.to_string(), 
        Some(512), 
        Some(0.7),
        Some("Qwen/QwQ-32B".to_string())).await.unwrap();

    let retrival_agent = retrival_agent_builder.tool(RetrivalTool).build();

    let agent_response = retrival_agent.prompt(prompt.clone()).await.unwrap_or_else(|e| {
        println!("An error occured! {e}");
        "Fail to process".to_string()
    });

    response = RetriveResponse {
        status: "success".to_string(),
        agent_response
    };

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
            .service(store_drift)
            .service(retrive_drift)
            .wrap(Logger::default())
            .wrap(Logger::new("%a"))
    })
    .workers(2)
    .bind((IPADDRESS, port))?
    .run()
    .await

}