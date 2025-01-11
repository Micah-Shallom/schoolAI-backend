use axum::{extract::State, Json};

use crate::{
    models::user::RegisterRequest,
    router::AppState,
    services::auth_service::register_user_service,
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
