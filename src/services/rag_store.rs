use std::sync::Arc;
use tokio::sync::Mutex;

use crate::{
    models::embeddings::{ActiveModel as EmbeddingActiveModel, Entity as Embedding},
    utils::errors::AppError,
};
use chrono::Utc;
use fastembed::TextEmbedding;
use sea_orm::{DatabaseConnection, EntityTrait, Set};
use usearch::{Index, IndexOptions, MetricKind, ScalarKind};
use uuid::Uuid;

pub struct RagStore {
    chunks: Vec<String>,
    embeddings: Vec<Vec<f32>>,
    index: Index,
}

impl RagStore {
    pub async fn new(
        db: DatabaseConnection,
        dimensions: usize,
    ) -> Result<Arc<Mutex<Self>>, AppError> {
        let capacity = dimensions;

        let options = IndexOptions {
            dimensions,
            metric: MetricKind::Cos,
            quantization: ScalarKind::F32,
            ..Default::default()
        };

        let index = Index::new(&options).map_err(|err| {
            AppError::InternalServerError(format!("Failed to create index: {}", err))
        })?;

        index.reserve(capacity).map_err(|err| {
            AppError::InternalServerError(format!("Failed to reserve capacity: {}", err))
        })?;

        let mut store = Self {
            chunks: Vec::new(),
            embeddings: Vec::new(),
            index,
        };

        store.index.reserve(capacity).map_err(|err| {
            AppError::InternalServerError(format!("Failed to reserve capacity: {}", err))
        })?;

        let cached = Embedding::find()
            .all(&db)
            .await
            .map_err(|e| AppError::InternalServerError(format!("Database error: {}", e)))?;

        if !cached.is_empty() {
            let chunks = cached.iter().map(|c| c.chunk.clone()).collect::<Vec<_>>();
            let embeddings = cached
                .iter()
                .map(|c| c.embedding.clone())
                .collect::<Vec<_>>();

            for (embedding, chunk) in embeddings.iter().zip(chunks.iter()) {
                store.add(chunk.clone(), embedding.clone()).map_err(|e| {
                    AppError::InternalServerError(format!("Failed to add embedding: {:?}", e))
                })?;
            }
        }

        println!("RagStore initialized with dimension {}", dimensions);
        println!("RagStore created successfully");

        Ok(Arc::new(Mutex::new(store)))
    }

    pub fn add(&mut self, chunk: String, embedding: Vec<f32>) -> Result<(), AppError> {
        if self.index.dimensions() != embedding.len() {
            return Err(AppError::InternalServerError(
                "Embedding dimensions mismatch".to_string(),
            ));
        }

        if embedding.iter().any(|&x| x.is_nan() || x.is_infinite()) {
            return Err(AppError::InternalServerError(
                "Embedding contains NaN or Inf values".to_string(),
            ));
        }

        let idx = self.chunks.len() as u64;
        self.index.add(idx, &embedding).map_err(|e| {
            AppError::InternalServerError(format!("Failed to add embedding: {}", e))
        })?;
        self.chunks.push(chunk);
        self.embeddings.push(embedding);
        Ok(())
    }

    pub async fn add_chunks_and_embeddings(
        &mut self,
        db: DatabaseConnection,
        filehash: &str,
        chunks: Vec<String>,
        embeddings: Vec<Vec<f32>>,
    ) -> Result<(), AppError> {
        if embeddings.len() != chunks.len() {
            return Err(AppError::InternalServerError(
                "Chunks and embeddings must have equal dimensions".to_string(),
            ));
        }

        if embeddings.is_empty() {
            return Err(AppError::InternalServerError(
                "Cannot create ragstore with empty embeddings".to_string(),
            ));
        }

        let dimension = embeddings[0].len();

        if embeddings.iter().any(|emb| emb.len() != dimension) {
            return Err(AppError::InternalServerError(
                "All embeddings must have the same dimension".to_string(),
            ));
        }

        self.index.reserve(chunks.len() as usize).map_err(|e| {
            AppError::InternalServerError(format!("Failed to reserve index capacity: {}", e))
        })?;

        let values = chunks
            .iter()
            .zip(embeddings.iter())
            .map(|(chunk, embedding)| EmbeddingActiveModel {
                id: Set(Uuid::new_v4()),
                file_hash: Set(filehash.to_string()),
                chunk: Set(chunk.clone()),
                embedding: Set(embedding.clone()),
                created_at: Set(Utc::now()),
            })
            .collect::<Vec<_>>();

        Embedding::insert_many(values)
            .exec(&db)
            .await
            .map_err(|e| {
                AppError::InternalServerError(format!("Failed to cache embeddings: {}", e))
            })?;

        for (chunk, embedding) in chunks.iter().zip(embeddings.iter()) {
            self.add(chunk.clone(), embedding.clone()).map_err(|e| {
                AppError::InternalServerError(format!("Failed to add chunk and embedding: {:?}", e))
            })?;
        }

        println!("RagStore created successfully");
        Ok(())
    }

    pub fn search(
        &self,
        query_embedding: &[f32],
        count: usize,
    ) -> Result<Vec<(String, f32)>, AppError> {
        let results = self
            .index
            .search(query_embedding, count)
            .map_err(|e| AppError::InternalServerError(format!("Search failed: {}", e)))?;
        let output = results
            .keys
            .iter()
            .zip(results.distances.iter())
            .filter_map(|(&idx, distance)| {
                let idx = idx as usize;
                self.chunks.get(idx).map(|chunk| (chunk.clone(), *distance))
            })
            .collect();

        Ok(output)
    }
}

// pub fn cosine_similarity(vec1: &[f32], vec2: &[f32]) -> f32 {
//     let dot_product = vec1
//         .iter()
//         .zip(vec2.iter())
//         .map(|(a, b)| a * b)
//         .sum::<f32>();
//     let norm1 = (vec1.iter().map(|x| x * x).sum::<f32>()).sqrt();
//     let norm2 = (vec2.iter().map(|x| x * x).sum::<f32>()).sqrt();
//     if norm1 == 0.0 || norm2 == 0.0 {
//         0.0
//     } else {
//         dot_product / (norm1 * norm2)
//     }
// }

pub fn retrieve_relevant_chunks(
    query: &str,
    store: &RagStore,
    model: &TextEmbedding,
) -> Result<Vec<String>, AppError> {
    let query_embedding = model
        .embed(vec![query.to_string()], None)
        .map_err(|e| AppError::InternalServerError(format!("Failed to embed query: {}", e)))?[0]
        .clone();

    //search for top-k relevant chunks (e.g. 5)
    let results = store.search(&query_embedding, 5)?;

    //extract chunks, ignoring distances for now
    let chunks = results
        .into_iter()
        .map(|(chunk, _distance)| chunk)
        .collect();

    Ok(chunks)
}
