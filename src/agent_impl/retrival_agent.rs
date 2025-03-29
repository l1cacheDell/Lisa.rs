use std::marker::PhantomData;

use crate::db_schemas::DriftBottle;
use tokio_rusqlite::Connection;
use rig::{
    agent::Agent, providers::openai::{Client, CompletionModel, EmbeddingModel}, vector_store::{self, VectorStoreIndex}, Embed, OneOrMany
};

use rig_sqlite::{SqliteVectorStore, SqliteVectorStoreTable};

pub struct RetrivalAgent<T: SqliteVectorStoreTable + 'static> {
    openai_api_key: String,
    base_url: String,
    sqlite_vec_db: String,
    model_name: String,
    embedding_model_name: String,
    embedding_ndim: usize,
    openai_client: Client,
    agent: Agent<CompletionModel>,
    phantom: PhantomData<T>
}

impl<T: SqliteVectorStoreTable + 'static> RetrivalAgent<T> {
    pub async fn new(
        openai_api_key: String,
        base_url: String,
        sqlite_vec_db: String,
        model_name: String,
        embedding_model_name: String,
        embedding_ndim: usize,
        sys_prompt: String,
        max_tokens: Option<u32>,
    ) -> Result<Self, anyhow::Error> {
        let conn = Connection::open(&sqlite_vec_db).await?;
        let openai_client = Client::from_url(&openai_api_key, &base_url);
        let embedding_model = openai_client.embedding_model_with_ndims(&embedding_model_name, embedding_ndim);
        let vector_store: SqliteVectorStore<EmbeddingModel, T> = SqliteVectorStore::new(conn, &embedding_model).await?;
        // let index = vector_store.index(embedding_model);
        let actual_max_tokens = max_tokens.unwrap_or(256);
        let agent = openai_client.agent(&model_name)
            .preamble(&sys_prompt)
            .max_tokens(actual_max_tokens.into())
            .dynamic_context(2, vector_store.index(embedding_model))  // `sample` means the number of top matched documents added to the agent context
            .build();

        Ok(Self {
            openai_api_key,
            base_url,
            sqlite_vec_db,
            model_name,
            embedding_model_name,
            embedding_ndim,
            openai_client,
            agent,
            phantom: PhantomData,
        })

    }

}