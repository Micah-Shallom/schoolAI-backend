use super::{extract::fetch_system_prompt, rag_generate::implement_rag};
use crate::{
    models::{
        embeddings::{Column, Entity as Embedding},
        features::{GeneratedResponse, McqGeneratorRequest},
    },
    services::{
        llm_service::run_prompt,
        rag_store::{retrieve_relevant_chunks, RagStore},
    },
    utils::errors::AppError,
};
use fastembed::TextEmbedding;
use openrouter_api::client;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use sha2::{Digest, Sha256};
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn mcq_service(
    db: &DatabaseConnection,
    req: McqGeneratorRequest,
    model: &TextEmbedding,
    rag_store: Arc<Mutex<RagStore>>,
    client: Arc<client::OpenRouterClient<client::Ready>>,
) -> Result<GeneratedResponse, AppError> {
    let sys_prompt = fetch_system_prompt("mcq").await.map_err(|e| {
        AppError::InternalServerError(format!("Failed to fetch system prompt: {:?}", e))
    })?;

    let base_prompt = format!(
        "{}\n\nGrade level: {}\n Number Of Questions: {}\nTopic: {}\nStandard objective: {}\nAdditional criteria: {}",
        sys_prompt,
        req.grade_level,
        req.number_of_questions,
        req.topic,
        req.standard_objective,
        req.additional_criteria.unwrap_or("None".to_string()),
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

    let prompt = format!(
        "{}\n\nRelevant context from uploaded content:\n{}",
        base_prompt, ragged_prompt
    );

    let response = run_prompt(&prompt, "qwen/qwq-32b:free", client)
        .await
        .map_err(|e| AppError::InternalServerError(format!("Failed to run prompt: {:?}", e)))?;

    Ok(response)
}
