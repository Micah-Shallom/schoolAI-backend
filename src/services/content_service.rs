use sea_orm::DatabaseConnection;

use crate::{models::features::AcademicContentRequest, utils::errors::AppError};

pub async fn content_service(
    _db: &DatabaseConnection,
    req: AcademicContentRequest,
    sys_prompt: String,
) -> Result<(), AppError> {
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
