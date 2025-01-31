use docx_rs::read_docx;
use serde_json::Value;
use std::{
    env,
    fs::{self, File},
    io::Read,
    path::Path,
};

use pdf_extract::extract_text;

pub async fn fetch_system_prompt(prompt_file_name: &str) -> Result<String, String> {
    let current_dir =
        env::current_dir().map_err(|err| format!("Failed to get current directory: {}", err))?;

    let base_dir = current_dir.join("src/services/prompts");

    let file_path = base_dir.join(format!("{}.txt", prompt_file_name));

    match fs::read_to_string(&file_path) {
        Ok(content) => Ok(content),
        Err(err) => {
            eprintln!(
                "error reading prompt file '{}': {}",
                file_path.display(),
                err
            ); //log the error
            Err(format!("prompt file '{}.txt' not found", prompt_file_name))
        }
    }
}

pub fn extract_from_file(file_path: &str) -> Result<String, String> {
    let path = Path::new(file_path);
    let extension = path
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.to_lowercase())
        .unwrap_or_default();

    match extension.as_str() {
        "pdf" => {
            // Extract text from PDF
            let text = extract_text(file_path)
                .map_err(|err| format!("Failed to extract text from PDF: {}", err))?;
            Ok(text)
        }
        "docx" => {
            // Extract text from DOCX
            let file_content = read_to_vec(file_path)?;
            let docx = read_docx(&file_content)
                .map_err(|err| format!("Failed to read DOCX file: {}", err))?;
            let text = parse_docx_json(&docx.json().to_string())?;
            Ok(text)
        }
        _ => Err(format!("Unsupported file format: {}", extension)),
    }
}

// Helper function to read a file into a Vec<u8>
fn read_to_vec(file_path: &str) -> Result<Vec<u8>, String> {
    let mut buffer = Vec::new();
    File::open(file_path)
        .map_err(|err| format!("Failed to open file '{}': {}", file_path, err))?
        .read_to_end(&mut buffer)
        .map_err(|err| format!("Failed to read file '{}': {}", file_path, err))?;
    Ok(buffer)
}

// Helper function to parse DOCX JSON and extract text
fn parse_docx_json(json: &str) -> Result<String, String> {
    let data: Value =
        serde_json::from_str(json).map_err(|err| format!("Failed to parse DOCX JSON: {}", err))?;
    let mut extracted_text = String::new();

    if let Some(children) = data["document"]["children"].as_array() {
        for child in children {
            extract_text_from_children(child, &mut extracted_text);
        }
    }

    Ok(extracted_text)
}

// Recursive function to extract text from DOCX JSON nodes
fn extract_text_from_children(node: &Value, extracted_text: &mut String) {
    if let Some(children) = node["data"]["children"].as_array() {
        for child in children {
            if child["type"] == "text" {
                if let Some(text) = child["data"]["text"].as_str() {
                    extracted_text.push_str(text);
                    extracted_text.push('\n'); // Add a newline for better formatting
                }
            } else {
                extract_text_from_children(child, extracted_text);
            }
        }
    }
}
