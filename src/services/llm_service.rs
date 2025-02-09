use chrono::Utc;
use openrouter_api::types::chat::{ChatCompletionRequest, Message};
use openrouter_api::OpenRouterClient;
use std::ops::Deref;
use std::sync::Arc;
use std::time::Instant;

use crate::models::features::GeneratedResponse;
use crate::utils::errors::AppError;

pub async fn run_prompt(
    prompt: &str,
    model: &str,
    client: Arc<OpenRouterClient<openrouter_api::Ready>>,
) -> Result<GeneratedResponse, AppError> {
    dotenvy::dotenv().ok();

    let request = ChatCompletionRequest {
        model: model.to_string(),
        messages: vec![Message {
            role: "user".to_string(),
            content: prompt.to_string(),
            name: None,
            tool_calls: None,
        }],
        stream: None,
        response_format: None,
        tools: None,
        provider: None,
        models: None,
        transforms: None,
    };

    let start_time = Instant::now();

    let response = client
        .deref()
        .chat()
        .map_err(|e| AppError::InternalServerError(e.to_string()))?
        .chat_completion(request)
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;
    let duration = start_time.elapsed();

    match response.choices.first() {
        Some(choice) => {
            println!("Chat response: {}", choice.message.content);
            let content = choice.message.content.clone();
            println!("LLM response received in {:2?}", duration);
            Ok(GeneratedResponse {
                content,
                generated_at: Utc::now(),
            })
        }
        None => {
            println!("LLM response received in {:2?}", duration);
            Err(AppError::InternalServerError(
                "No response from LLM".to_string(),
            ))
        }
    }
}
