use chrono::Utc;
use openrouter_api::types::chat::{ChatCompletionRequest, Message};
use openrouter_api::OpenRouterClient;
use std::time::{Duration, Instant};

use crate::models::features::AcademicContentResponse;
use crate::utils::errors::AppError;

pub async fn run_prompt(_prompt: &str, _model: &str) -> Result<AcademicContentResponse, AppError> {
    dotenvy::dotenv().ok();


    let request = ChatCompletionRequest {
        model: "qwen/qwen-2-7b-instruct:free".to_string(),
        messages: vec![Message {
            role: "user".to_string(),
            content: "generate a curriculum for a 8th grade class for computer science".to_string(),
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
        .chat_completion(request)
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;
    let duration = start_time.elapsed();

    match response.choices.first() {
        Some(choice) => {
            println!("Chat response: {}", choice.message.content);
            let content = choice.message.content.clone();
            println!("LLM response received in {:2?}", duration);
            Ok(AcademicContentResponse {
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
