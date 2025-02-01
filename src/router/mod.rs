pub mod auth_routes;
pub mod feature_routes;
pub mod test_routes;

use std::sync::Arc;
use tokio::sync::Mutex;

use crate::{
    config::jwt::JwtConfig, models::embeddings, services::rag_store::RagStore,
    utils::errors::AppError,
};
use axum::{response::IntoResponse, Router};
use fastembed::TextEmbedding;
use sea_orm::DatabaseConnection;

#[derive(Clone)]
pub struct AppState {
    pub db: DatabaseConnection,
    pub embedding_model: Arc<TextEmbedding>,
    pub jwt_config: JwtConfig,
    pub version: String,
    pub rag_store: Arc<Mutex<RagStore>>,
}

pub fn create_router(
    db: DatabaseConnection,
    jwt_config: JwtConfig,
    embedding_model: Arc<TextEmbedding>,
    rag_store: Arc<Mutex<RagStore>>,
) -> Router {
    let state = AppState {
        db: db.clone(),
        jwt_config: jwt_config.clone(),
        version: String::from("/api/v1"),
        embedding_model,
        rag_store,
    };

    Router::new()
        .merge(test_routes::routes(state.clone()))
        .merge(auth_routes::routes(state.clone()))
        .merge(feature_routes::routes(state.clone()))
        .fallback(fallback_handler)
        .with_state(state)
}

async fn fallback_handler() -> impl IntoResponse {
    AppError::NotFound("Route not found".to_string())
}
