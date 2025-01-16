use sea_orm::DatabaseConnection;

use crate::{models::features::AcademicContentRequest, utils::errors::AppError};

use super::extract::fetch_system_prompt;

pub async fn content_service(
    _db: &DatabaseConnection,
    req: AcademicContentRequest,
) -> Result<(), AppError> {
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
