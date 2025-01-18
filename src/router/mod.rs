pub mod auth_routes;
pub mod feature_routes;
pub mod test_routes;

use std::sync::Arc;

use crate::{config::jwt::JwtConfig, utils::errors::AppError};
use axum::{response::IntoResponse, Router};
use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};
use sea_orm::DatabaseConnection;

#[derive(Clone)]
pub struct AppState {
    pub db: DatabaseConnection,
    pub embedding_model: Arc<TextEmbedding>,
    pub jwt_config: JwtConfig,
    pub version: String,
}

pub fn create_router(db: DatabaseConnection, jwt_config: JwtConfig) -> Router {
    let state = AppState {
        db,
        jwt_config: jwt_config.clone(),
        version: String::from("/api/v1"),
        embedding_model: Arc::new(
            TextEmbedding::try_new(
                InitOptions::new(EmbeddingModel::AllMiniLML6V2).with_show_download_progress(true),
            )
            .expect("Failed to initialize TextEmbedding"),
        ),
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
