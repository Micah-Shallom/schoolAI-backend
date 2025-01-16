use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct AcademicContentRequest {
    pub grade_level: String,
    pub content_type: String,
    pub text_length: String,
    pub topic: String,
    pub standard_objective: String,
    pub additional_criteria: Option<String>,
    pub uploaded_content: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct AcademicContentResponse {
    pub content: String,
    pub generated_at: DateTime<Utc>,
}
