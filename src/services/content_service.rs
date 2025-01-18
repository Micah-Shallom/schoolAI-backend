use std::ops::Deref;

use chrono::Utc;
use fastembed::TextEmbedding;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, ActiveValue::Set};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::models::embeddings::{
    self, ActiveModel as EmbeddingActiveModel, Column, Entity as Embedding,
};
use crate::services::rag_generate::{chunk_text, generate_text_embeddings};
use crate::services::rag_store::{retrieve_relevant_chunks, RagStore};
use crate::{models::features::AcademicContentRequest, utils::errors::AppError};

use super::{extract::fetch_system_prompt, rag_generate::implement_rag};

pub async fn content_service(
    db: &DatabaseConnection,
    req: AcademicContentRequest,
    model: &TextEmbedding,
) -> Result<(), AppError> {
    let sys_prompt = fetch_system_prompt("academic_content").await.map_err(|e| {
        AppError::InternalServerError(format!("Failed to fetch system prompt: {:?}", e))
    })?;

    let base_prompt = format!(
        "{}\n\nGrade level: {}\nLength: {}\nTopic: {}\nStandard objective: {}\nAdditional criteria: {}",
            sys_prompt,
            req.grade_level,
            req.text_length,
            req.topic,
            req.standard_objective,
            req.additional_criteria.unwrap_or("None".to_string())
    );

    let final_prompt = if let Some(uploaded_content) = req.uploaded_content.as_deref() {
        let mut hasher = Sha256::new();
        hasher.update(uploaded_content);
        let file_hash = format!("{:x}", hasher.finalize());

        let cached = Embedding::find()
            .filter(Column::FileHash.eq(file_hash.clone()))
            .all(db)
            .await
            .map_err(|e| AppError::InternalServerError(format!("Database error: {}", e)))?;

        let store = if !cached.is_empty() {
            let chunks = cached.iter().map(|c| c.chunk.clone()).collect::<Vec<_>>();
            let embeddings = cached
                .iter()
                .map(|c| c.embedding.clone())
                .collect::<Vec<_>>();
            RagStore::from_chunks_and_embeddings(chunks, embeddings)
        } else {
            let chunks = chunk_text(uploaded_content, 500);
            let embeddings = generate_text_embeddings(chunks.clone(), model).map_err(|e| {
                AppError::InternalServerError(format!("Failed to generate embeddings: {}", e))
            })?;

            let values = chunks
                .clone()
                .iter()
                .zip(embeddings.iter())
                .map(|(chunk, embedding)| EmbeddingActiveModel {
                    id: Set(Uuid::new_v4()),
                    file_hash: Set(file_hash.clone()),
                    chunk: Set(chunk.clone()),
                    embedding: Set(embedding.clone()),
                    created_at: Set(Utc::now()),
                    ..Default::default()
                })
                .collect::<Vec<_>>();

            Embedding::insert_many(values).exec(db).await.map_err(|e| {
                AppError::InternalServerError(format!("Failed to cache embeddings: {}", e))
            })?;


            RagStore::from_chunks_and_embeddings(chunks.clone(), embeddings)
        };

        let relevant_chunks =
            retrieve_relevant_chunks(&req.topic, &store, &model, 3).map_err(|e| {
                AppError::InternalServerError(format!("Failed to retrieve chunks: {}", e))
            })?;
        let context = relevant_chunks.join("\n");

        println!("context======================================================================================== {:?}", context);

        format!(
            "{}\n\nRelevant context from uploaded content: {}",
            base_prompt, context
        )
    } else {
        return Err(AppError::BadRequest(
            "Uploaded content is required to create a RagStore.".to_string(),
        ));
    };

    // println!("Final Prompt {:?}", final_prompt);

    Ok(())
}
