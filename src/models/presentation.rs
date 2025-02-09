use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Serialize, Clone)]
pub struct PresentationGeneratorRequest {
    pub grade_level: String,
    pub number_of_slides: i32,
    pub topic: String,
    pub standard_objective: String,
    pub additional_criteria: Option<String>,
    pub uploaded_content: Option<String>,
}
#[derive(Debug, Serialize)]
pub struct PresentationGeneratorResponse {
    pub presentation_url: String,
    pub ppt_id: String,
    pub pdf_url: String,
    pub presentation_details: Option<PresentationDetails>,
    pub generated_at: chrono::DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct PresentationDetails {
    pub presentation_title: String,
    pub presentation_subtitle: String,
    pub image_search: String,
    pub slides: Vec<Slide>,
}

#[derive(Debug, Serialize)]
pub struct Slide {
    pub title: String,
    pub subtitle: String,
    pub image_search: String,
    pub content: Vec<SlideContent>,
}

#[derive(Debug, Serialize)]
pub struct SlideContent {
    pub title: String,
    pub description: String,
}

#[derive(Serialize)]
pub struct MagicSlidesRequest {
    pub topic: String,
    #[serde(rename = "extraInfoSource")]
    pub extra_info_source: String,
    pub email: String,
    #[serde(rename = "accessId")]
    pub access_id: String,
    pub template: String,
    pub language: String,
    #[serde(rename = "slideCount")]
    pub slide_count: i32,
    #[serde(rename = "aiImages")]
    pub ai_images: bool,
    #[serde(rename = "imageForEachSlide")]
    pub image_for_each_slide: bool,
    #[serde(rename = "googleImage")]
    pub google_image: bool,
    #[serde(rename = "googleText")]
    pub google_text: bool,
    pub model: String,
    #[serde(rename = "presentationFor")]
    pub presentation_for: String,
}

// Response struct for MagicSlidesAPI
#[derive(Deserialize)]
pub struct MagicSlidesApiResponse {
    pub status: String,
    pub message: String,
    pub data: MagicSlidesData,
}

#[derive(Deserialize)]
pub struct MagicSlidesData {
    pub url: String,
    pub json: Option<MagicSlidesJson>,
    #[serde(rename = "pptId")]
    pub ppt_id: String,
    #[serde(rename = "pdfUrl")]
    pub pdf_url: String,
}

#[derive(Deserialize)]
pub struct MagicSlidesJson {
    #[serde(rename = "presentationTitle")]
    pub presentation_title: String,
    #[serde(rename = "presentationSubtitle")]
    pub presentation_subtitle: String,
    #[serde(rename = "imageSearch")]
    pub image_search: String,
    pub slides: Vec<MagicSlidesSlide>,
}

#[derive(Deserialize)]
pub struct MagicSlidesSlide {
    pub title: String,
    pub subtitle: String,
    #[serde(rename = "imageSearch")]
    pub image_search: String,
    pub content: Vec<MagicSlidesSlideContent>,
}

#[derive(Deserialize)]
pub struct MagicSlidesSlideContent {
    pub title: String,
    pub description: String,
}
