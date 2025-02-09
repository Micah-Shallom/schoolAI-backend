use axum::{
    extract::{Multipart, State},
    Json,
};

use serde_json::Value;

use tempfile::NamedTempFile;
use tokio::fs;

use crate::{
    models::features::McqGeneratorRequest,
    router::AppState,
    services::{extract::extract_from_file, mcq_service::mcq_service},
    utils::errors::AppError,
};

use std::path::Path;

pub async fn generate_mcq_content(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<Json<Value>, AppError> {
    let mut request = McqGeneratorRequest {
        grade_level: String::new(),
        topic: String::new(),
        number_of_questions: 0,
        standard_objective: String::new(),
        additional_criteria: None,
        uploaded_content: None,
    };

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::BadRequest(e.to_string()))?
    {
        let name = field.name().unwrap_or("").to_string();
        match name.as_str() {
            "grade_level" => {
                request.grade_level = field
                    .text()
                    .await
                    .map_err(|e| AppError::BadRequest(e.to_string()))?
            }
            "number_of_questions" => {
                request.number_of_questions = field
                    .text()
                    .await
                    .map_err(|e| AppError::BadRequest(e.to_string()))?
                    .parse::<i32>()
                    .map_err(|e| AppError::BadRequest(e.to_string()))?;
            }
            "topic" => {
                request.topic = field
                    .text()
                    .await
                    .map_err(|e| AppError::BadRequest(e.to_string()))?
            }
            "standard_objective" => {
                request.standard_objective = field
                    .text()
                    .await
                    .map_err(|e| AppError::BadRequest(e.to_string()))?
            }
            "additional_criteria" => {
                request.additional_criteria = Some(
                    field
                        .text()
                        .await
                        .map_err(|e| AppError::BadRequest(e.to_string()))?,
                )
            }
            "uploaded_content" => {
                let file_name = field.file_name().map(|name| name.to_string());

                if file_name.clone().unwrap() == "" {
                    AppError::InternalServerError("Empty file".to_string());
                }

                let data = &field
                    .bytes()
                    .await
                    .map_err(|e| AppError::BadRequest(e.to_string()))?;

                let file_extension = file_name
                    .as_deref()
                    .and_then(|name| Path::new(name).extension())
                    .and_then(|ext| ext.to_str())
                    .unwrap_or("tmp");

                let temp_file = NamedTempFile::new().map_err(|e| {
                    AppError::InternalServerError(format!("Failed to create temp file: {}", e))
                })?;

                let temp_file_path = temp_file.path().with_extension(file_extension);

                std::fs::rename(temp_file.path(), &temp_file_path).map_err(|e| {
                    AppError::InternalServerError(format!(
                        "Failed to rename temp file to include extension: {}",
                        e
                    ))
                })?;

                fs::write(&temp_file_path, &data).await.map_err(|e| {
                    AppError::InternalServerError(format!("Failed to write temp file: {}", e))
                })?;

                let extracted_text =
                    extract_from_file(temp_file_path.to_str().ok_or_else(|| {
                        AppError::InternalServerError(
                            "Failed to convert temp file path to string".to_string(),
                        )
                    })?);

                request.uploaded_content = Some(extracted_text.unwrap());
            }
            _ => {}
        }
    }

    let response = mcq_service(
        &state.db,
        request,
        state.embedding_model.as_ref(),
        state.rag_store,
        state.client,
    )
    .await?;

    Ok(Json(serde_json::json!({
        "status": "success",
        "data": response
    })))
}
