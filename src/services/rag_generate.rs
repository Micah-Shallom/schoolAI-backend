use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};
use std::vec;

pub fn implement_rag(text: &str, model: &TextEmbedding) -> Result<Vec<Vec<f32>>, String> {
    let chunk_size = 500;

    let chunks = chunk_text(text, chunk_size);

    let embeddings = generate_text_embeddings(chunks, model)?;

    Ok(embeddings)
}

pub fn chunk_text(text: &str, chunk_size: usize) -> Vec<String> {
    text.split("\n\n")
        .flat_map(|paragraph| {
            if paragraph.len() <= chunk_size {
                vec![paragraph.to_string()]
            } else {
                paragraph
                    .chars()
                    .collect::<Vec<_>>()
                    .chunks(chunk_size)
                    .map(|chunk| chunk.iter().collect())
                    .collect::<Vec<_>>()
            }
        })
        .filter(|chunk| !chunk.trim().is_empty())
        .collect()
}

pub fn generate_text_embeddings(
    documents: Vec<String>,
    model: &TextEmbedding,
) -> Result<Vec<Vec<f32>>, String> {
    // Generate embeddings with the default batch size, 256
    let embeddings = model
        .embed(documents, None)
        .map_err(|e| format!("Failed to generate embeddings: {}", e))?;

    // println!("Embeddings length: {}", embeddings.len()); // -> Embeddings length: 4
    // println!("Embedding dimension: {}", embeddings[0].len()); // -> Embedding dimension: 384

    Ok(embeddings)
}
