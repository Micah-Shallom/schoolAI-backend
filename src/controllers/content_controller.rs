use axum::{extract::State, Json};

use crate::{models::features::AcademicContentRequest, router::AppState, utils::errors::AppError};

pub async fn generate_academic_content (
    State(state): State<AppState>,
    Json(request): Json<AcademicContentRequest>,
) -> Result<Json<serde_json::Value>, AppError>  {

    println!("Request: {:?}", request);

    Ok(())
}