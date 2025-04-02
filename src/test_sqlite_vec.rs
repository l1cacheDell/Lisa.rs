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
struct Document {
    id: String,
    #[embed]
    content: String,
}

impl SqliteVectorStoreTable for Document {
    fn name() -> &'static str {
        "documents"
    }

    fn schema() -> Vec<Column> {
        vec![
            Column::new("id", "TEXT PRIMARY KEY"),
            Column::new("content", "TEXT"),
        ]
    }

    fn id(&self) -> String {
        self.id.clone()
    }

    fn column_values(&self) -> Vec<(&'static str, Box<dyn ColumnValue>)> {
        vec![
            ("id", Box::new(self.id.clone())),
            ("content", Box::new(self.content.clone())),
        ]
    }
}

pub async fn launch_sqlite_vec(openai_client: &Client, embedding_model: &str) -> Result<(), anyhow::Error> {

    // Initialize OpenAI client

    // Initialize the `sqlite-vec`extension
    // See: https://alexgarcia.xyz/sqlite-vec/rust.html
    unsafe {
        sqlite3_auto_extension(Some(std::mem::transmute(sqlite3_vec_init as *const ())));
    }

    // Initialize SQLite connection
    let conn = Connection::open("data/vector_store.db").await?;

    // Select the embedding model and generate our embeddings
    let bge_v3_ndim = 1024;
    let model = openai_client.embedding_model_with_ndims(embedding_model, bge_v3_ndim);

    println!("The ndim is: {:?}", model.ndims());

    // Initialize SQLite vector store
    let vector_store: SqliteVectorStore<rig::providers::openai::EmbeddingModel, Document> = SqliteVectorStore::new(conn, &model).await?;

    let documents = vec![
        Document {
            id: "doc0".to_string(),
            content: "Definition of a *flurbo*: A flurbo is a green alien that lives on cold planets".to_string(),
        },
        Document {
            id: "doc1".to_string(), 
            content: "Definition of a *glarb-glarb*: A glarb-glarb is a ancient tool used by the ancestors of the inhabitants of planet Jiro to farm the land.".to_string(),
        },
        Document {
            id: "doc2".to_string(),
            content: "Definition of a *linglingdong*: A term used by inhabitants of the far side of the moon to describe humans.".to_string(),
        },
    ];

    let embeddings = EmbeddingsBuilder::new(model.clone())
        .documents(documents)?
        .build()
        .await?;

    // Add embeddings to vector store
    // let store_res = vector_store.add_rows(embeddings)
    //                      .await
    //                      .unwrap_or(-1);
    // println!("The storage result is: {store_res}");

    // Create a vector index on our vector store
    let index = vector_store.index(model);
    // let index2 = SqliteVectorIndex::new(model, &vector_store);

    // Query the index
    let results = index
        .top_n::<Document>("What is a linglingdong, about the moon?", 1)
        .await?
        .into_iter()
        .map(|(score, id, doc)| (score, id, doc))
        .collect::<Vec<_>>();

    println!("Results: {:?}", results);

    let id_results = index
        .top_n_ids("What is a linglingdong?", 1)
        .await?
        .into_iter()
        .collect::<Vec<_>>();

    println!("ID results: {:?}", id_results);

    Ok(())
}