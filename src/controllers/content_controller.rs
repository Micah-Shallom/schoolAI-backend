use axum::{extract::State, Json};

use crate::{
    models::features::AcademicContentRequest,
    router::AppState,
    services::{content_service::content_service, system_prompt::fetch_system_prompt},
    utils::{errors::AppError, response::success_response},
};

pub async fn generate_academic_content(
    State(state): State<AppState>,
    Json(request): Json<AcademicContentRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let sys_prompt = fetch_system_prompt("academic_content").await.unwrap();

    let response = content_service(&state.db, request, sys_prompt).await?;

    let rd = serde_json::to_value(response)
        .map_err(|e| AppError::InternalServerError(format!("JSON serialization error: {}", e)))?;

    Ok(Json(success_response(rd)))
}
