// use std::os::linux::raw;

use rig::providers::openai;
use dotenvy::dotenv;
use rig::completion::Prompt;
use rig::streaming::{StreamingPrompt, StreamingChoice};

pub mod db_schemas;
pub mod agent_impl;
pub mod request_model;
pub mod test_sqlite_vec;
pub mod aptos_utils;

use request_model::{ChatRequest, GeneralReponse, RetriveRequest, RetriveResponse, GradeBottleRequest, GradeBottleResponse};
use agent_impl::{RetrivalAgent, prompt_hub, RetrivalTool};
use aptos_utils::verify_tx;

use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder, Error};
use actix_web::middleware::Logger;
use actix_cors::Cors;
use env_logger::Env;
use rusqlite::ffi::sqlite3_auto_extension;
use sqlite_vec::sqlite3_vec_init;

use futures::{future::ok, stream::once};
use futures::StreamExt; // 关键引入



#[get("/")]
async fn entrance() -> actix_web::Result<impl Responder> {
    Ok(web::Json(GeneralReponse {
        status: "Welcome to emptylab!".to_string()
    }))
}

#[get("/api/ping")]
async fn ping() -> actix_web::Result<impl Responder> {
    Ok(web::Json(GeneralReponse {
        status: "pong".to_string()
    }))
}

// this API will be streaming response
#[post("/api/chat")]
async fn chat(json: web::Json<ChatRequest>) -> HttpResponse {
    let _wallet = &json.wallet;
    let prompt = &json.content;

    let sys_prompt = prompt_hub::CHAT_AGENT_SYS_PROMPT;

    let mut max_tokens = 64;
    if db_schemas::count_sequence_len(prompt) > 64 {
        max_tokens = 128;
    }

    // let chat_agent = RetrivalAgent::new(
    //     sys_prompt.to_string(), 
    //     Some(max_tokens), 
    //     Some(0.9), 
    //     Some(1)).await.unwrap();

    let chat_agent_builder = RetrivalAgent::new_builder(
        sys_prompt.to_string(), 
        Some(max_tokens), 
        Some(0.9), 
        Some("deepseek-ai/DeepSeek-V3".to_string()));

    let chat_agent = chat_agent_builder.await.unwrap().build();

    // TODO: we need to use `stream_chat` interface, and figure out one way to store the chat history of a single user.
    let raw_response = chat_agent.stream_prompt(prompt)
        .await;

    match raw_response {
        Ok(stream) => {
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
        },
        Err(e) => {
            eprintln!("Full error: {:#?}", e);  // 打印完整错误结构
            HttpResponse::InternalServerError().body(format!("Backend error: {}", e))
        }
    }
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
    let mut response = GradeBottleResponse {
        status: "OK".to_string(),
        score: 98
    };

    // 1. verify the tx_hash
    let valid = verify_tx(tx_hash).await.unwrap_or(false);
    if !valid {
        response = GradeBottleResponse {
            status: "transaction invalid".to_string(),
            score: -1
        };
        return Ok(web::Json(response));
    }
    // 2. save these content to vec db, using function

    // 3. grade this content, return score.


    Ok(web::Json(response))
}

#[get("/api/retrive_drift")]
async fn retrive_drift(json: web::Json<RetriveRequest>) -> actix_web::Result<impl Responder> {
    let _wallet = &json.wallet;
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

    const IPADDRESS: &str = "0.0.0.0";
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
            .wrap(cors)
            .service(entrance)
            .service(ping)
            .service(chat)
            .service(store_drift)
            .service(retrive_drift)
            .wrap(Logger::default())
            .wrap(Logger::new("%a"))
    })
    .workers(4)
    .bind((IPADDRESS, port))?
    .run()
    .await

}