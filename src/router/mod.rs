pub mod auth_routes;
pub mod test_routes;

use crate::config::jwt::JwtConfig;
use axum::Router;
use sea_orm::DatabaseConnection;

#[derive(Clone)]
pub struct AppState {
    pub db: DatabaseConnection,
    pub jwt_config: JwtConfig,
}

pub fn create_router(db: DatabaseConnection, jwt_config: JwtConfig) -> Router {
    let state = AppState {
        db,
        jwt_config: jwt_config.clone(),
    };

    Router::new()
        .merge(test_routes::routes(state.clone()))
        .merge(auth_routes::routes(state.clone()))
        .with_state(state)
}
