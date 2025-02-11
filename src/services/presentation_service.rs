use super::{extract::fetch_system_prompt, rag_generate::implement_rag};
use crate::models::presentation::MagicSlidesRequest;
use crate::{
    config::{self, config::Configuration},
    models::{
        embeddings::{Column, Entity as Embedding},
        features::GeneratedResponse,
        presentation::{
            MagicSlidesApiResponse, PresentationDetails, PresentationGeneratorRequest,
            PresentationGeneratorResponse, Slide, SlideContent,
        },
    },
    services::rag_store::{retrieve_relevant_chunks, RagStore},
    utils::errors::AppError,
};
use chrono::Utc;
use fastembed::TextEmbedding;
use openrouter_api::client;
use reqwest::Client;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use sha2::{Digest, Sha256};
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn presentation_service(
    db: &DatabaseConnection,
    req: PresentationGeneratorRequest,
    model: &TextEmbedding,
    rag_store: Arc<Mutex<RagStore>>,
    client: Arc<client::OpenRouterClient<client::Ready>>,
) -> Result<GeneratedResponse, AppError> {
    let sys_prompt = fetch_system_prompt("presentation").await.map_err(|e| {
        AppError::InternalServerError(format!("Failed to fetch system prompt: {:?}", e))
    })?;

    let base_prompt = format!(
        "{}\n\nGrade level: {}\n Number Of Slides: {}\nTopic: {}\nStandard objective: {}\nAdditional criteria: {}",
        sys_prompt,
        req.grade_level,
        req.number_of_slides,
        req.topic,
        req.standard_objective,
        req.additional_criteria.clone().unwrap_or("None".to_string()),
    );

    let ragged_prompt = if let Some(uploaded_content) = req.uploaded_content.as_deref() {
        let mut hasher = Sha256::new();
        hasher.update(uploaded_content);
        let file_hash = format!("{:x}", hasher.finalize());

        let cached = Embedding::find()
            .filter(Column::FileHash.eq(file_hash.clone()))
            .all(db)
            .await
            .map_err(|e| AppError::InternalServerError(format!("Database error: {}", e)))?;

        let mut store = rag_store.lock().await;

        if cached.is_empty() {
            println!("No cached embeddings found for file hash: {}", file_hash);

            let (chunks, embeddings) = implement_rag(uploaded_content, model).map_err(|e| {
                AppError::InternalServerError(format!("Failed to implement RAG: {}", e))
            })?;

            if chunks.is_empty() || embeddings.is_empty() {
                return Err(AppError::BadRequest(
                    "No chunks or embeddings generated.".to_string(),
                ));
            }

            println!("Adding {} chunks and embeddings to the store", chunks.len());

            store
                .add_chunks_and_embeddings(db.clone(), &file_hash, chunks, embeddings)
                .await
                .map_err(|e| {
                    AppError::InternalServerError(format!(
                        "Failed to add chunks and embeddings: {:?}",
                        e
                    ))
                })?;
        } else {
            println!("Using cached embeddings for file hash: {}", file_hash);

            let chunks = cached.iter().map(|c| c.chunk.clone()).collect::<Vec<_>>();
            let embeddings = cached
                .iter()
                .map(|c| c.embedding.clone())
                .collect::<Vec<_>>();

            for (chunk, embedding) in chunks.into_iter().zip(embeddings.into_iter()) {
                store.add(chunk, embedding).map_err(|e| {
                    AppError::InternalServerError(format!(
                        "Failed to add cached embedding: {:?}",
                        e
                    ))
                })?;
            }
        }

        let relevant_chunks = retrieve_relevant_chunks(&req.topic, &store, model).map_err(|e| {
            AppError::InternalServerError(format!("Failed to retrieve chunks: {:?}", e))
        })?;
        let context = relevant_chunks.join("\n");

        context
    } else {
        return Err(AppError::BadRequest(
            "Uploaded content is required to create a RagStore.".to_string(),
        ));
    };

    let extra_info_source = format!(
        "{}\n\nRelevant context from uploaded content:\n{}",
        base_prompt, ragged_prompt
    );

    let magic_response = send_external_request(&req, extra_info_source).await?;

    let response = GeneratedResponse {
        content: serde_json::to_string(&magic_response).map_err(|e| {
            AppError::InternalServerError(format!("Failed to serialize response: {}", e))
        })?,
        generated_at: Utc::now(),
    };

    Ok(response)
}

// will write a robust service later on to handle all types of external request with all request and response types
pub async fn send_external_request(
    req: &PresentationGeneratorRequest,
    prompt: String,
) -> Result<PresentationGeneratorResponse, AppError> {
    let configuration = Configuration::init_env();

    let magic_slides_req = MagicSlidesRequest {
        topic: req.topic.clone(),
        extra_info_source: prompt,
        email: "micahshallom@gmail.com".to_string(),
        access_id: configuration.magic_slide_access_id,
        template: "bullet-point1".to_string(),
        language: "en".to_string(),
        slide_count: req.number_of_slides,
        ai_images: false,
        image_for_each_slide: true,
        google_image: false,
        google_text: false,
        model: "gpt-4".to_string(),
        presentation_for: format!("grade {} students", req.grade_level),
    };

    // Send request to MagicSlidesAPI
    let client = Client::new();
    let response = client
        .post(configuration.magic_slide_base_url)
        .json(&magic_slides_req)
        .send()
        .await
        .map_err(|e| {
            AppError::InternalServerError(format!("MagicSlides API request failed: {}", e))
        })?;

    // Check response status
    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(AppError::InternalServerError(format!(
            "MagicSlides API returned error: status {}, body {}",
            status, body
        )));
    }

    // Parse response
    let magic_slides_resp: MagicSlidesApiResponse = response.json().await.map_err(|e| {
        AppError::InternalServerError(format!("Failed to parse MagicSlides response: {}", e))
    })?;

    let presentation_details = magic_slides_resp.data.json.map(|json| PresentationDetails {
        presentation_title: json.presentation_title,
        presentation_subtitle: json.presentation_subtitle,
        image_search: json.image_search,
        slides: json
            .slides
            .into_iter()
            .map(|s| Slide {
                title: s.title,
                subtitle: s.subtitle,
                image_search: s.image_search,
                content: s
                    .content
                    .into_iter()
                    .map(|c| SlideContent {
                        title: c.title,
                        description: c.description,
                    })
                    .collect(),
            })
            .collect(),
    });

    // Construct response
    Ok(PresentationGeneratorResponse {
        presentation_url: magic_slides_resp.data.url,
        ppt_id: magic_slides_resp.data.ppt_id,
        pdf_url: magic_slides_resp.data.pdf_url,
        presentation_details,
        generated_at: Utc::now(),
    })
}
