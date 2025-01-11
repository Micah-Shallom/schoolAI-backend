use crate::middleware::auth::AuthenticatedUser;
use axum::extract::Extension;
use axum::Json;
use serde::Serialize;

#[derive(Serialize)]
pub struct HelloResponse {
    message: String,
}

pub async fn hello_world() -> Json<HelloResponse> {
    Json(HelloResponse {
        message: "Hello, World".to_string(),
    })
}

// Simplified signature with imports
pub async fn protected_hello(Extension(user): Extension<AuthenticatedUser>) -> Json<HelloResponse> {
    Json(HelloResponse {
        message: format!("Hello, {}! You are authenticated.", user.email),
    })
}
