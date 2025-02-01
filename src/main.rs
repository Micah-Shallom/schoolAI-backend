mod config;
mod controllers;
mod middleware;
mod models;
mod router;
mod services;
mod utils;

use config::jwt::JwtConfig;
use migration::{Migrator, MigratorTrait};
use services::rag_store::RagStore;

use crate::config::config::Configuration;
use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};
use std::sync::Arc;

#[tokio::main]
async fn main() {
    let configuration = Configuration::init_env();

    //Run migrations before establishing connection
    println!("Running database migrations...");
    let db_url = &configuration.database_url;

    let migrations_connection = sea_orm::Database::connect(db_url)
        .await
        .expect("Failed to connect to database for migrations");

    match Migrator::up(&migrations_connection, None).await {
        Ok(_) => {
            println!("Migrations completed successfully");
        }
        Err(e) => {
            eprintln!("Failed to run migrations: {}", e);
            return;
        }
    }

    //establish db connection
    let db = match configuration.establish_connection().await {
        Ok(conn) => {
            println!("Database connection established successfully");
            conn
        }
        Err(e) => {
            eprintln!("Failed to establish database connection: {}", e);
            return;
        }
    };

    let rag_store = RagStore::new(db.clone(), 384)
        .await
        .expect("Failed to create RagStore");

    //create jwt config
    let jwt_config = JwtConfig::new(
        configuration.jwt_secret.clone(),
        configuration.jwt_expiration,
    );

    // Initialize the embedding model
    let embedding_model = Arc::new(
        TextEmbedding::try_new(
            InitOptions::new(EmbeddingModel::AllMiniLML6V2).with_show_download_progress(true),
        )
        .expect("Failed to initialize TextEmbedding"),
    );

    let app = router::create_router(db, jwt_config, embedding_model, rag_store); // share db connection with all handlers

    let port = configuration.server_port;
    let addr = format!("0.0.0.0:{}", port);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();

    println!("Server listening on http://{}", addr);

    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}
