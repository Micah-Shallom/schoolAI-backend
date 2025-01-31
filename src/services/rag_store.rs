use fastembed::TextEmbedding;
use usearch::Index;

use crate::utils::errors::AppError;

pub struct RagStore {
    chunks: Vec<String>,
    embeddings: Vec<Vec<f32>>,
    index: Index,

}

impl RagStore {
    pub fn new(dimensions: usize) -> Result<Self,AppError> {
        let index = Index::new(&usearch::IndexOptions {
            dimensions,
            metric: usearch::MetricKind::Cos,
            quantization: usearch::ScalarKind::F32,
            ..Default::default()
        }).map_err(|err| AppError::InternalServerError(format!("Failed to create index {}", err)));

        Ok(Self {
            chunks: Vec::new(),
            embeddings: Vec::new(),
            index: index.unwrap(),
        })
    }

    pub fn add(&mut self, chunk: String, embedding: Vec<f32>) -> Result<(), AppError> {
        if self.index.dimensions() != embedding.len() {
            return Err(AppError::InternalServerError("Embedding dimensions mismatch".to_string()))
        }

        let idx = self.chunks.len() as u64;
        self.index.add(idx, &embedding)
            .map_err(|e| AppError::InternalServerError(format!("Failed to add embedding: {}", e)));
        self.chunks.push(chunk);
        self.embeddings.push(embedding);
        Ok(())
    }

    pub fn from_chunks_and_embeddings(chunks: Vec<String>, embeddings: Vec<Vec<f32>>) -> Result<Self, AppError> {
        
        if embeddings.len() != chunks.len() {
            return Err(AppError::InternalServerError("Chunks and embeddings must have equal dimenstions".to_string()));
        }

        if embeddings.is_empty() {
            return Err(AppError::InternalServerError("Cannot create ragstore with empty embeddings".to_string()));
        }

        let dimension = embeddings[0].len();

        if embeddings.iter().any(|emb| emb.len() != dimension) {
            return Err(AppError::InternalServerError("All embeddings must have the same dimension".to_string()));
        }

        let mut store = Self::new(dimension)?;
        for (chunk, emb) in chunks.iter().zip(embeddings.iter()) {
            store.add(chunk.clone(), emb.clone())?;
        }
        Ok(store)
    }

    pub fn search(&self, query_embedding: &[f32], count: usize) -> Result<Vec<(String, f32)>, AppError>{
        let results = self.index.search(query_embedding, count).map_err(|e| AppError::InternalServerError(format!("Search failed: {}", e)))?;
        let output = results
            .keys
            .iter()
            .zip(results.distances.iter())
            .filter_map(|(&idx, distance)| {
                let idx = idx as usize;
                self.chunks
                    .get(idx)
                    .map(|chunk| (chunk.clone(), *distance))
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
