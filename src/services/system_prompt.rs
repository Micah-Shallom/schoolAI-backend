use std::{fs, path::Path};


use crate::utils::errors::AppError;

pub async fn fetch_system_prompt(prompt_name: &str) -> Result<String, AppError> {
    let base_dir = "./prompts";

    let file_path = Path::new(base_dir).join(format!("{}.txt", prompt_name));

    let content = fs::read_to_string(&file_path)
        .map_err(|_| AppError::NotFound(format!("Prompt file '{}' not found", prompt_name)))?;

    Ok(content)
}