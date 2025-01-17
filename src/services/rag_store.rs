use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};

pub struct RagStore {
    chunks: Vec<String>,
    embeddings: Vec<Vec<f32>>,
}

impl RagStore {
    pub fn new() -> Self {
        Self {
            chunks: Vec::new(),
            embeddings: Vec::new(),
        }
    }

    pub fn add(&mut self, chunk: String, embedding: Vec<f32>) {
        self.chunks.push(chunk);
        self.embeddings.push(embedding);
    }

    pub fn from_chunks_and_embeddings(chunks: Vec<String>, embeddings: Vec<Vec<f32>>) -> Self {
        assert_eq!(chunks.len(), embeddings.len(), "Chunks and embeddings must have the same length");
        Self {chunks, embeddings}
    }
}


pub fn cosine_similarity(vec1: &[f32], vec2: &[f32]) -> f32 {
    let dot_product = vec1.iter().zip(vec2.iter()).map(|(a, b)| a * b).sum::<f32>();
    let norm1 = (vec1.iter().map(|x| x * x).sum::<f32>()).sqrt();
    let norm2 = (vec2.iter().map(|x| x * x).sum::<f32>()).sqrt();
    if norm1 == 0.0 || norm2 == 0.0 { 0.0 } else { dot_product / (norm1 * norm2) }
}

pub fn retrieve_relevant_chunks(
    query: &str,
    store: &RagStore,
    model: &TextEmbedding,
    top_k: usize,
) -> Result<Vec<String>, String> {
    let query_embedding = model.embed(vec![query.to_string()], None)
        .map_err(|e| format!("Failed to embed query: {}", e))?[0].clone();
    let mut similarities: Vec<(usize, f32)> = store.embeddings
        .iter()
        .enumerate()
        .map(|(i, emb)| (i, cosine_similarity(&query_embedding, emb)))
        .collect();
    similarities.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    let top_indices = similarities.into_iter().take(top_k).map(|(i, _)| i);
    Ok(top_indices.map(|i| store.chunks[i].clone()).collect())
}