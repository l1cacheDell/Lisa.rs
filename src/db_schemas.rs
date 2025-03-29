use rig::{
    embeddings::EmbeddingsBuilder,
    providers::openai::Client,
    vector_store::VectorStoreIndex,
    Embed, OneOrMany,
};
use rig_sqlite::{Column, ColumnValue, SqliteVectorIndex, SqliteVectorStore, SqliteVectorStoreTable};
use rusqlite::ffi::sqlite3_auto_extension;
use serde::Deserialize;
use sqlite_vec::sqlite3_vec_init;
use tokio_rusqlite::Connection;
use rig::embeddings::EmbeddingModel;

use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Embed, Clone, Debug, Deserialize)]
pub struct DriftBottle {
    id: String,
    wallet: String,
    title: String,
    #[embed]
    content: String,
}

impl SqliteVectorStoreTable for DriftBottle {
    fn name() -> &'static str {
        "drift_bottles"
    }

    fn schema() -> Vec<Column> {
        vec![
            Column::new("id", "TEXT PRIMARY KEY"),
            Column::new("wallet", "TEXT"),
            Column::new("title", "TEXT"),
            Column::new("content", "TEXT"),
        ]
    }

    fn id(&self) -> String {
        self.id.clone()
    }

    fn column_values(&self) -> Vec<(&'static str, Box<dyn ColumnValue>)> {
        vec![
            ("id", Box::new(self.id.clone())),
            ("wallet", Box::new(self.wallet.clone())),
            ("title", Box::new(self.title.clone())),
            ("content", Box::new(self.content.clone())),
        ]
    }
}

static GLOBAL_ID: AtomicUsize = AtomicUsize::new(0);

pub fn get_next_id() -> usize {
    GLOBAL_ID.fetch_add(1, Ordering::Relaxed) // 原子操作，线程安全
}

const DOCUMENT_STRIDE: usize = 510;

pub async fn store_drift_vec(wallet: &str, title: &str, content: &str) -> Result<(), anyhow::Error>{
    // load from env vars
    let db_path = std::env::var("DB_PATH").expect("DB_PATH not set");
    let openai_api_key = std::env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY not set");
    let base_url: String = std::env::var("BASE_URL").expect("BASE_URL not set");
    let embedding_model_name: String = std::env::var("EMBEDDING_MODEL_NAME").expect("MODEL_NAME not set");
    let embedding_ndim = std::env::var("EMBEDDING_MODEL_NDIM")
        .unwrap_or_else(|_| "1024".to_string())
        .parse()?;

    // start building
    // check if this document has already been stored
    let conn = Connection::open(&db_path).await?;
    let wallet_clone = wallet.to_string();
    let title_clone = format!("{}-{}", title.to_string(), 0);

    let stored_repeated = conn.call(|conn| {
        let mut stmt = conn.prepare(
            "SELECT * FROM drift_bottles WHERE wallet = ? AND title = ?"
        )?;

        let searched_docs = stmt.query_map([wallet_clone, title_clone], |row| {
            Ok(DriftBottle {
                id: row.get(0)?,
                wallet: row.get(1)?,
                title: row.get(2)?,
                content: row.get(3)?
            })
        })?
        .collect::<std::result::Result<Vec<DriftBottle>, rusqlite::Error>>()?;

        Ok(searched_docs)
    })
    .await?;

    if stored_repeated.len() > 0 {
        return Err(anyhow::Error::msg("This document has already been stored"));
    }

    // store this doc
    let openai_client = Client::from_url(&openai_api_key, &base_url);
    let embedding_model = openai_client.embedding_model_with_ndims(&embedding_model_name, embedding_ndim);
    let vector_store: SqliteVectorStore<rig::providers::openai::EmbeddingModel, DriftBottle> = SqliteVectorStore::new(conn, &embedding_model).await?;

    let mut docs: Vec<DriftBottle> = Vec::new();
    let mut len_cnt = 0;
    let mut passage_cnt = 0;

    loop {
        if len_cnt >= content.len() {
            break;
        }
        let end = std::cmp::min(len_cnt + DOCUMENT_STRIDE, content.len());
        let content_part = &content[len_cnt..end];
        let new_id = get_next_id();
        docs.push(DriftBottle {
            id: new_id.to_string(), 
            wallet: wallet.to_string(), 
            title: format!("{}-{}", title, passage_cnt), 
            content: content_part.to_string() 
        });
        len_cnt = end;
        passage_cnt += 1;
    }

    let embeddings = EmbeddingsBuilder::new(embedding_model)
        .documents(docs)?
        .build()
        .await?;

    // save it to db
    vector_store.add_rows(embeddings).await?;

    Ok(())
}