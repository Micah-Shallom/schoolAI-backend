use sea_orm::DatabaseConnection;

use crate::{models::features::AcademicContentRequest, utils::errors::AppError};

use super::{extract::fetch_system_prompt, rag_generate::implement_rag};

pub async fn content_service(
    _db: &DatabaseConnection,
    req: AcademicContentRequest,
) -> Result<(), AppError> {

    let uploaded_content = req
        .uploaded_content
        .as_deref()
        .ok_or_else(|| AppError::BadRequest("Uploaded content is missing".to_string()))?;

    let embeddings = implement_rag(uploaded_content)
        .map_err(|e| {
            AppError::InternalServerError(format!("Failed to generate embeddings: {}", e))
        });

    println!("embeddings {:?}", embeddings);


    let sys_prompt = fetch_system_prompt("academic_content").await.map_err(|e| {
        AppError::InternalServerError(format!("Failed to fetch system prompt: {:?}", e))
    })?;

    let prompt = format!(
        "{}\n\nGrade level: {}\nLength: {}\nTopic: {}\nStandard objective: {}\nAdditional criteria: {}",
            sys_prompt,
            req.grade_level,
            req.text_length,
            req.topic,
            req.standard_objective,
            req.additional_criteria.unwrap_or("None".to_string())
    );

    Ok(())
}
