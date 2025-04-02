use dotenvy::dotenv;
use rig::providers::openai::Client;
use rig::completion::Prompt;
use rig::streaming::{StreamingPrompt, StreamingChoice};

pub mod db_schemas;
pub mod agent_impl;
pub mod request_model;
pub mod test_sqlite_vec;

use request_model::{ChatRequest, ChatResponse, GeneralReponse, RetriveRequest, RetriveResponse, GradeBottleRequest, GradeBottleResponse};
use agent_impl::{RetrivalAgent, prompt_hub, RetrivalTool};

use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder, Error};
use actix_web::middleware::Logger;
use actix_cors::Cors;
use env_logger::Env;
use rusqlite::ffi::sqlite3_auto_extension;
use sqlite_vec::sqlite3_vec_init;

use futures::{future::ok, stream::once};
use futures::{StreamExt}; // 关键引入

#[get("/api/ping")]
async fn ping() -> actix_web::Result<impl Responder> {
    Ok(web::Json(GeneralReponse {
        status: "pong".to_string()
    }))
}

// this API will be streaming response
#[post("/api/chat")]
async fn chat(json: web::Json<ChatRequest>) -> HttpResponse {
    let wallet = &json.wallet;
    let prompt = &json.content;

    let sys_prompt = prompt_hub::CHAT_AGENT_SYS_PROMPT;

    let chat_agent = RetrivalAgent::new(
        sys_prompt.to_string(), 
        Some(128), 
        Some(0.9), 
        Some(1)).await.unwrap();

    // TODO: we need to use `stream_chat`` interface, and figure out one way to store the chat history of a single user.
    let stream = chat_agent.stream_prompt(prompt)
        .await
        .unwrap();

    let converted_stream = stream
        .map(|result| { // 使用 StreamExt 的 map
            match result {
                Ok(choice) => match choice {
                    StreamingChoice::Message(text) => Ok(web::Bytes::from(text)),
                    StreamingChoice::ToolCall(_, _, _) => 
                        Err(actix_web::error::ErrorBadRequest("Tool calls not supported")),
                },
                Err(e) => 
                    Err(actix_web::error::ErrorInternalServerError(e)),
            }
        })
        .boxed(); // 统一流类型

    HttpResponse::Ok()
        .content_type("application/json")
        .streaming(converted_stream)
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
        response = GeneralReponse {
            status: format!("Error: {}", e)
        };
    });
    Ok(web::Json(response))
}

#[get("/api/grade_drift")]
async fn grade_drift(json: web::Json<GradeBottleRequest>) -> actix_web::Result<impl Responder> {
    let title = &json.title;
    let content = &json.content;
    let tx_hash = &json.tx_hash;

    // 1. verify the tx_hash

    // 2. save these content to vec db, using function

    // 3. grade this content, return score.

    let mut response = GradeBottleResponse {
        status: "OK".to_string(),
        score: 98
    };

    Ok(web::Json(response))
}

#[get("/api/retrive_drift")]
async fn retrive_drift(json: web::Json<RetriveRequest>) -> actix_web::Result<impl Responder> {
    let wallet = &json.wallet;
    let prompt = &json.content;

    let mut response = RetriveResponse {
        status: "success".to_string(),
        retrive_results: Vec::new(),
    };

    let valid_tx: bool = true;
    if !valid_tx {
        response = RetriveResponse {
            status: "Fail to response".to_string(),
            retrive_results: Vec::new(),
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

    let doc_info = db_schemas::parse_markdown_text(&agent_response).unwrap_or_else(|e| {
        println!("An Error occured during parsing: {}", e);
        Vec::new()
    });

    if doc_info.len() == 0 {
        response = RetriveResponse {
            status: "Sorry, we haven't found any similar exprience as you have now.".to_string(),
            retrive_results: doc_info
        };

        return Ok(web::Json(response));
    }

    response = RetriveResponse {
        status: "success".to_string(),
        retrive_results: doc_info
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
        let cors = Cors::default()
            .allow_any_origin()
            .allowed_methods(vec!["GET", "POST"])
            .allow_any_header()
            .max_age(3600);

        App::new()
            .service(ping)
            .service(chat)
            .service(store_drift)
            .service(retrive_drift)
            .wrap(Logger::default())
            .wrap(Logger::new("%a"))
            .wrap(cors)
    })
    .workers(2)
    .bind((IPADDRESS, port))?
    .run()
    .await

}