use std::time::{Duration, Instant};

use openrouter_api::{OpenRouterClient, Result};
use openrouter_api::types::chat::{ChatCompletionRequest, Message};


#[tokio::main]
async fn main() -> Result<()> {
   dotenvy::dotenv().ok(); 

   let api_key = std::env::var("OPENROUTER_API_KEY")
      .expect("OPENROUTER_API_KEY must be set");

   let client = OpenRouterClient::new()
      .with_base_url("https://openrouter.ai/api/v1/")?
      .with_timeout(Duration::from_secs(500))
      .with_api_key(api_key)?;

   let request = ChatCompletionRequest{
      model: "qwen/qwen-2-7b-instruct:free".to_string(),
      messages: vec![
         Message{
            role:"user".to_string(),
            content:"generate a curriculum for a 8th grade class for computer science".to_string(),
            name: None,
            tool_calls:None,
         }
      ],
      stream: None,
      response_format: None,
      tools: None,
      provider: None,
      models: None,
      transforms: None,
   };

   let start_time =  Instant::now();

   let response = client.chat_completion(request).await?;
   let duration = start_time.elapsed();
   
   if let Some(choice) = response.choices.first() {
      println!("Chat response: {}", choice.message.content);
   }

   println!("LLM response received in {:2?}", duration);

   Ok(())
}