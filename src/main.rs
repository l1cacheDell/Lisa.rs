use dotenvy::dotenv;
use rig::providers::openai::Client;
use rig::completion::Prompt;

pub mod db_schemas;
pub mod agent_impl;
pub mod web_model;
pub mod test_sqlite_vec;

use web_model::{ChatRequest, ChatResponse, GeneralReponse};
use agent_impl::RetrivalTool;

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
    let user_id = &json.user_id;
    let prompt = &json.content;

    let mut response = ChatResponse {
        status: "success".to_string(),
        agent_response: "I don't have a response for that yet.".to_string(),
    };

    let vcdb_from_env = db_schemas::VectorDBFromEnv::new()
        .await
        .unwrap();

    let openai_client = Client::from_url(&vcdb_from_env.openai_api_key, &vcdb_from_env.base_url);

    let chat_agent = openai_client.agent(&vcdb_from_env.model_name)
        .preamble(r#"You are a sexy & charming & cool Metherland-Japan hybrid girl like Lucy in Cyberpunk: Edge Runner, and you served in a club as a Bartender. 
The customers will sometimes talk to you and share their emotional story. Your job is to talk to them in deep, and provide guidance.

Besides, you can use an agent tool which named 'search_related_story', this is a function used to search related stories from the vector database, if you want to search for experience related to the customer you are serving now, just use tool call this function, and it will return the finding result. If there is indeed some related stories, feel free to use these materials to offer a better talking experience to user.

After retrival the story, avoid sending this story to customer directly. Instead, you should use this material as additional resource, reflect, and offer your own words to compose an answer, and talk to user.
        "#)
        .tool(RetrivalTool)
        .temperature(0.8)
        .max_tokens(256)
        .build();

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