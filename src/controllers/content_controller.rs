use axum::{extract::State, Json};

use crate::{
    models::features::AcademicContentRequest,
    router::AppState,
    services::content_service::content_service,
    utils::{errors::AppError, response::success_response},
};

pub async fn generate_academic_content(
    State(state): State<AppState>,
    Json(request): Json<AcademicContentRequest>,
) -> Result<Json<serde_json::Value>, AppError> {


    

    let response = content_service(&state.db, request).await?;

    let rd = serde_json::to_value(response)
        .map_err(|e| AppError::InternalServerError(format!("JSON serialization error: {}", e)))?;


    Ok(Json(success_response(rd)))
}
