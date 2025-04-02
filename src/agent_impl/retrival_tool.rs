use serde::{Deserialize, Serialize};
use serde_json::json;

use tokio_rusqlite::Connection;
use rig::{
    agent::{Agent, AgentBuilder}, providers::openai::{Client, CompletionModel, EmbeddingModel}, 
    vector_store::{self, VectorStoreIndex}
};
use rig::completion::ToolDefinition;
use rig::tool::Tool;

use rig_sqlite::{SqliteVectorStore, SqliteVectorStoreTable};
use crate::db_schemas::{DriftBottle, VectorDBFromEnv};

// sqlite vec, and retrival tool
// during retrival process, we will only retrive the 

#[derive(Deserialize)]
pub struct RetrivalArgs {
    topic_sentence: String,
    user: Option<String>,
}

#[derive(Serialize)]
pub struct RetrivalOption {
    pub user: String,
    pub title: String,
    pub summary: String,
}

#[derive(Debug, thiserror::Error)]
pub enum RetrivalError {
    #[error("Vector index failed: {0}")]
    VectorIndex(String),
    #[error("Vector store failed: {0}")]
    VectorStore(String),
    #[error("Missing API key: {0}")]
    MissingApiKey(String),
    #[error("Connection error: {0}")]
    VectorConn(String)
}
pub struct RetrivalTool;

impl Tool for RetrivalTool {
    const NAME: &'static str = "search_related_story";

    type Args = RetrivalArgs;
    type Output = String;
    type Error = RetrivalError;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: Self::NAME.to_string(),
            description: "Search for embeddings-highly-similiar stories in vector database.".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "topic_sentence": {"type": "string", "description": "The topic and summarized sentence of user's inquiry to best match related stories. (e.g. 'Blue emotion, regretful loss of a beloved, looking for comfort and support.')"},
                    "user": {"type": "string", "description": "Optional param, a user or wallet address to filter the be-searched stories. (e.g. '0xc4d6C15db36b92dC4776d2Ead5dd31Df86202A3B')"}
                },
                "required": ["topic_sentence"]
            })
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let vcdb_from_env = VectorDBFromEnv::new().await.map_err(|_| {
            RetrivalError::MissingApiKey("Env load error".to_string())
        })?;
        let conn = Connection::open(&vcdb_from_env.db_path)
            .await
            .map_err(|e| {
                RetrivalError::VectorConn(e.to_string())
            })?;
        
        let openai_client = Client::from_url(&vcdb_from_env.openai_api_key, &vcdb_from_env.base_url);
        let embedding_model = openai_client.embedding_model_with_ndims(&vcdb_from_env.embedding_model_name, 
            vcdb_from_env.embedding_ndim);
        let vector_store: SqliteVectorStore<rig::providers::openai::EmbeddingModel, DriftBottle> = SqliteVectorStore::new(conn, &embedding_model)
            .await
            .map_err(|e| {
                RetrivalError::VectorStore(e.to_string())
            })?;
        let vector_index = vector_store.index(embedding_model);

        let results: Vec<(f64, String, DriftBottle)> = vector_index
            .top_n::<DriftBottle>(&args.topic_sentence, 2)
            .await
            .map_err(|e| {
                RetrivalError::VectorIndex(e.to_string())
            })?
            .into_iter()
            .map(|(score, id, doc)| {
                (score, id, doc)
            })
            .collect::<Vec<_>>();

        let mut output = String::new();
        for (_i, doc) in results.iter().enumerate() {
            println!("Doc sim: {}", doc.0);
            if doc.0 > 0.7 {
                output.push_str(&format!("**id**: {}\n**User**: {}\n**title**: {}\n**content**: {}", doc.2.id, doc.2.wallet, doc.2.title, doc.2.content));
                output.push_str("\n\n\n");
            }
        }

        if output.len() == 0 {
            return Ok("No highly similar passages about this topic.".to_string());
        }

        Ok(output)
    }
}

pub struct RetrivalAgent;

impl RetrivalAgent {
    pub async fn new(
        sys_prompt: String,
        max_tokens: Option<u32>,
        temperature: Option<f32>,
        top_sample: Option<u8>
    ) -> Result<Agent<CompletionModel>, anyhow::Error> {
        let vcdb_from_env = VectorDBFromEnv::new().await.map_err(|_| {
            RetrivalError::MissingApiKey("Env load error".to_string())
        })?;
        let conn = Connection::open(&vcdb_from_env.db_path)
            .await
            .map_err(|e| {
                RetrivalError::VectorConn(e.to_string())
            })?;
        
        let openai_client = Client::from_url(&vcdb_from_env.openai_api_key, &vcdb_from_env.base_url);
        let embedding_model = openai_client.embedding_model_with_ndims(&vcdb_from_env.embedding_model_name, 
            vcdb_from_env.embedding_ndim);
        let vector_store: SqliteVectorStore<rig::providers::openai::EmbeddingModel, DriftBottle> = SqliteVectorStore::new(conn, &embedding_model)
            .await
            .map_err(|e| {
                RetrivalError::VectorStore(e.to_string())
            })?;
        let vector_index = vector_store.index(embedding_model);

        let actual_max_tokens = max_tokens.unwrap_or(256);
        let actual_temperature = temperature.unwrap_or(0.7);
        let actual_top_n_sample: u8 = top_sample.unwrap_or(2);
        let agent = openai_client.agent(&vcdb_from_env.model_name)
            .preamble(&sys_prompt)
            .max_tokens(actual_max_tokens.into())
            .temperature(actual_temperature.into())
            .dynamic_context(actual_top_n_sample.into(), 
            vector_index)  // `sample` means the number of top matched documents added to the agent context
            .build();

        Ok(agent)

    }

    pub async fn new_builder(
        sys_prompt: String,
        max_tokens: Option<u32>,
        temperature: Option<f32>,
        model_name: Option<String>
    ) -> Result<AgentBuilder<CompletionModel>, anyhow::Error> {
        let vcdb_from_env = VectorDBFromEnv::new().await.map_err(|_| {
            RetrivalError::MissingApiKey("Env load error".to_string())
        })?;
        
        let openai_client = Client::from_url(&vcdb_from_env.openai_api_key, &vcdb_from_env.base_url);

        let actual_max_tokens = max_tokens.unwrap_or(256);
        let actual_temperature = temperature.unwrap_or(0.7);
        let actual_model_name = model_name.unwrap_or(vcdb_from_env.model_name.to_string());

        let agent = openai_client.agent(&actual_model_name)
            .preamble(&sys_prompt)
            .max_tokens(actual_max_tokens.into())
            .temperature(actual_temperature.into());

        Ok(agent)
    }

}

