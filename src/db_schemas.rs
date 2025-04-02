use rig::{
    embeddings::EmbeddingsBuilder,
    providers::openai::Client,
    Embed
};
use rig_sqlite::{Column, ColumnValue, SqliteVectorIndex, SqliteVectorStore, SqliteVectorStoreTable};
use serde::{Deserialize, Serialize};
use tokio_rusqlite::Connection;
use regex::Regex;

use std::sync::atomic::{AtomicUsize, Ordering};

//
//  ==================== Low-Level Database Schema ====================
//
#[derive(Embed, Clone, Debug, Deserialize)]
pub struct DriftBottle {
    pub id: String,
    pub wallet: String,
    pub title: String,
    #[embed]
    pub content: String,
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

//
//  ==================== High-Level Database Schema ====================
//
// #[derive(Embed, Clone, Debug, Deserialize)]
// pub struct BottleSummary {
//     id: String,
//     wallet: String,
//     title: String,
//     keywords: String,
//     #[embed]
//     summary: String,
// }

// impl SqliteVectorStoreTable for BottleSummary {
//     fn name() -> &'static str {
//         "bottle_summaries"
//     }

//     fn schema() -> Vec<Column> {
//         vec![
//             Column::new("id", "TEXT PRIMARY KEY"),
//             Column::new("wallet", "TEXT"),
//             Column::new("title", "TEXT"),
//             Column::new("keywords", "TEXT"),
//             Column::new("summary", "TEXT"),
//         ]
//     }

//     fn id(&self) -> String {
//         self.id.clone()
//     }

//     fn column_values(&self) -> Vec<(&'static str, Box<dyn ColumnValue>)> {
//         vec![
//             ("id", Box::new(self.id.clone())),
//             ("wallet", Box::new(self.wallet.clone())),
//             ("title", Box::new(self.title.clone())),
//             ("keywords", Box::new(self.keywords.clone())),
//             ("summary", Box::new(self.summary.clone())),
//         ]
//     }
// }

// public storage zone
static GLOBAL_ID: AtomicUsize = AtomicUsize::new(0);

pub fn get_next_id() -> usize {
    GLOBAL_ID.fetch_add(1, Ordering::Relaxed) // 原子操作，线程安全
}

const DOCUMENT_STRIDE: usize = 510;

pub struct VectorDBFromEnv {
    pub db_path: String,
    pub openai_api_key: String,
    pub base_url: String,
    pub embedding_model_name: String,
    pub embedding_ndim: usize,
    pub model_name: String
}

impl VectorDBFromEnv {
    pub async fn new() -> Result<Self, anyhow::Error> {
        let db_path = std::env::var("DB_PATH").unwrap_or("data/vector_store.db".to_string());
        let openai_api_key = std::env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY not set");
        let base_url: String = std::env::var("BASE_URL").expect("BASE_URL not set");
        let embedding_model_name: String = std::env::var("EMBEDDING_MODEL_NAME").expect("MODEL_NAME not set");
        let embedding_ndim = std::env::var("EMBEDDING_MODEL_NDIM")
            .unwrap_or_else(|_| "1024".to_string())
            .parse()?;
        let model_name = std::env::var("MODEL_NAME").expect("MODEL_NAME not set");

        Ok(Self {
            db_path,
            openai_api_key,
            base_url,
            embedding_model_name,
            embedding_ndim,
            model_name
        })
    }
}

pub async fn store_drift_vec(wallet: &str, title: &str, content: &str) -> Result<(), anyhow::Error>{
    // load from env vars

    let vcdb_from_env = VectorDBFromEnv::new().await?;

    // start building
    // check if this document has already been stored
    let conn = Connection::open(&vcdb_from_env.db_path).await?;
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
    .await
    .unwrap_or_else(|e| {
        if e.to_string().contains("no such table") {
            return Vec::new();  // handle error: table not created yet, so continue to store. The table and schema will be created later.
        }
        panic!("Error querying database: {}", e);
    });

    if stored_repeated.len() > 0 {
        return Err(anyhow::Error::msg("This document has already been stored"));
    }

    // store this doc
    let openai_client = Client::from_url(&vcdb_from_env.openai_api_key, &vcdb_from_env.base_url);
    let embedding_model = openai_client.embedding_model_with_ndims(&vcdb_from_env.embedding_model_name, 
        vcdb_from_env.embedding_ndim);
    let vector_store: SqliteVectorStore<rig::providers::openai::EmbeddingModel, DriftBottle> = SqliteVectorStore::new(conn, &embedding_model).await?;

    // Notice: the length of the passage, is not the length of the string.
    // The granularity of the passage is word-level, not character-level.
    let mut docs: Vec<DriftBottle> = Vec::new();
    let mut start = 0;

    let word_re = Regex::new(r"\b[\w\p{P}]+\b").unwrap();
    let words: Vec<&str> = word_re.find_iter(content).map(|mat| mat.as_str()).collect();

    while start < words.len() {
        let end = std::cmp::min(start + DOCUMENT_STRIDE, words.len());
        let content_part = &words[start..end];
        let new_id = get_next_id();

        docs.push(DriftBottle {
            id: new_id.to_string(), 
            wallet: wallet.to_string(), 
            title: format!("{}-{}", title, docs.len()), // title-0, title-1, ...
            content: content_part.join(" ")
        });

        start = end;
    }

    let embeddings = EmbeddingsBuilder::new(embedding_model)
        .documents(docs)?
        .build()
        .await?;

    // save it to db
    vector_store.add_rows(embeddings).await?;

    Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DocInfo {
    pub id: String,
    pub user: String,
    pub title: String,
    pub content: String,
}

pub fn parse_markdown_text(text: &str) -> Result<Vec<DocInfo>, String> {
    let mut cleaned_chunk = text.replace("\\n", "\n");
    cleaned_chunk = cleaned_chunk.replace("\"", "");
    let vec_texts = cleaned_chunk.split("\n\n\n");

    let id_re = Regex::new(r"\*\*id\*\*: (.+)").unwrap();
    let user_re = Regex::new(r"\*\*User\*\*: (.+)").unwrap();
    let title_re = Regex::new(r"\*\*title\*\*: (.+)").unwrap();
    let content_re = Regex::new(r"(?s)\*\*content\*\*: (.+)").unwrap();  // 使用 (?s) 让 . 匹配换行

    let mut results: Vec<DocInfo> = Vec::new();

    for chunk in vec_texts {
        if !chunk.contains("**id**") || !chunk.contains("**User**") || !chunk.contains("**title**") || !chunk.contains("**content**") {
            continue;
        }
        let id = id_re.captures(chunk)
            .and_then(|c| c.get(1)).map(|m| m.as_str().to_string())
            .ok_or("ID 匹配失败")?;
        let user = user_re.captures(chunk)
            .and_then(|c| c.get(1)).map(|m| m.as_str().to_string())
            .ok_or("user 匹配失败")?;
        let title = title_re.captures(chunk)
            .and_then(|c| c.get(1)).map(|m| m.as_str().to_string())
            .ok_or("title 匹配失败")?;
        let content = content_re.captures(chunk)
            .and_then(|c| c.get(1)).map(|m| m.as_str().to_string())
            .ok_or("content 匹配失败")?;
        
        results.push(DocInfo {
            id,
            user,
            title,
            content
        })
    }

    Ok(results)
}