use dotenvy::dotenv;
use rig::providers::openai::Client;

pub mod test_completion;
use crate::test_completion::create_completion;

pub mod test_sqlite_vec;
use crate::test_sqlite_vec::launch_sqlite_vec;

pub mod db_schemas;
pub mod agent_impl;

use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use actix_web::middleware::Logger;
use env_logger::Env;

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[post("/echo")]
async fn echo(req_body: String) -> impl Responder {
    HttpResponse::Ok().body(req_body)
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
    let _response2 = launch_sqlite_vec(&openai_client, &embedding_model_name).await.unwrap();

    env_logger::init_from_env(Env::default().default_filter_or("info"));

    HttpServer::new(|| {
        App::new()
            .service(hello)
            .service(echo)
            .wrap(Logger::default())
            .wrap(Logger::new("%a"))
    })
    .workers(4)
    .bind((IPADDRESS, PORT))?
    .run()
    .await

}