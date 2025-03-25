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

