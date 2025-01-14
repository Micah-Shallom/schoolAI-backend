use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct AcademicContentRequest {
    grade_level: String,
    content_type: String,
    text_length: String,
    topic: String,
    standard_objective: String,
    additional_criteria: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct AcademicContentResponse {
    pub content: String,
    pub generated_at: DateTime<Utc>,
}
