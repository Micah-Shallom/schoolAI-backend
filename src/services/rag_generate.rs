use fastembed::TextEmbedding;
use text_splitter::MarkdownSplitter;

pub fn implement_rag(
    text: &str,
    model: &TextEmbedding,
) -> Result<(Vec<String>, Vec<Vec<f32>>), String> {
    let chunk_size = 1000..2000; // Range for flexibility, adjust based on testing

    // Use MarkdownSplitter for semantic chunking
    let splitter = MarkdownSplitter::new(chunk_size);
    let chunks: Vec<String> = splitter
        .chunks(text)
        .map(|s| s.to_string())
        .filter(|s| s.len() > 50) // Skip very short chunks (e.g., headers)
        .collect();

    if chunks.is_empty() {
        return Err("No valid chunks generated".to_string());
    }

    let embeddings = generate_text_embeddings(chunks.clone(), model)?;

    Ok((chunks, embeddings))
}

pub fn generate_text_embeddings(
    documents: Vec<String>,
    model: &TextEmbedding,
) -> Result<Vec<Vec<f32>>, String> {
    let embeddings = model
        .embed(documents, None)
        .map_err(|e| format!("Failed to generate embeddings: {}", e))?;

    // Validate and normalize embeddings
    let normalized_embeddings: Vec<Vec<f32>> = embeddings
        .into_iter()
        .map(|mut emb| {
            if emb.iter().any(|&x| x.is_nan() || x.is_infinite()) {
                return Err("Embedding contains NaN or Inf".to_string());
            }
            let norm = emb.iter().map(|&x| x * x).sum::<f32>().sqrt();
            if norm > 0.0 {
                emb.iter_mut().for_each(|x| *x /= norm);
            }
            Ok(emb)
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(normalized_embeddings)
}
