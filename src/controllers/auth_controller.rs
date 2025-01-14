use axum::{extract::State, Json};
use serde_json::json;

use crate::{
    models::user::{LoginRequest, LogoutRequest, RegisterRequest},
    router::AppState,
    services::auth_service::{login_user_service, logout_user_service, register_user_service},
    utils::{errors::AppError, response::success_response},
};

pub async fn register_user(
    State(state): State<AppState>,
    Json(request): Json<RegisterRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let response = register_user_service(&state.db, &state.jwt_config, request).await?;

    let rd = serde_json::to_value(response)
        .map_err(|e| AppError::InternalServerError(format!("JSON serialization error: {}", e)))?;

    Ok(Json(success_response(rd)))
}

pub async fn login_user(
    State(state): State<AppState>,
    Json(request): Json<LoginRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let response = login_user_service(&state.db, &state.jwt_config, request).await?;

    let rd = serde_json::to_value(response)
        .map_err(|e| AppError::InternalServerError(format!("JSON serialization error: {}", e)))?;

    Ok(Json(success_response(rd)))
}

pub async fn logout_user(
    State(state): State<AppState>,
    Json(request): Json<LogoutRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    logout_user_service(&state.db, &state.jwt_config, request).await?;

    let response = json!({
        "message": "Logout successful"
    });

    Ok(Json(success_response(response)))
}
