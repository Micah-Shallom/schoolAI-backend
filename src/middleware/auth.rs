use crate::{
    models::blacklist::{Column, Entity as Blacklist},
    router::AppState,
    utils::errors::AppError,
};
use axum::{
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
    Json,
};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct AuthenticatedUser {
    user_id: Uuid,
    pub email: String,
    is_admin: bool,
}

pub async fn auth_middleware(
    State(app_state): State<AppState>,
    mut request: Request<axum::body::Body>,
    next: Next,
) -> Result<Response, AppError> {
    // Extract jwt_config from the AppState
    let jwt_config = &app_state.jwt_config;

    // Get Authorization header
    let auth_header = request
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .ok_or(AppError::Unauthorized(
            "Authorization header missing or invalid".to_string(),
        ))?;

    // Extract token
    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or(AppError::Unauthorized(
            "Bearer token missing or invalid".to_string(),
        ))?;

    let is_blacklisted = Blacklist::find()
        .filter(Column::Token.eq(token))
        .one(&app_state.db)
        .await
        .map_err(|e| AppError::InternalServerError(format!("Database query error {e}")))?;

    if is_blacklisted.is_some() {
        return Err(AppError::Unauthorized(
            "Token has been blacklisted".to_string(),
        ));
    }

    // Validate token
    let claims = jwt_config
        .validate_token(token)
        .map_err(|_| AppError::Unauthorized("Invalid or expired token".to_string()))?;

    // Create AuthenticatedUser and insert into request extensions
    let user = AuthenticatedUser {
        user_id: claims.sub,
        email: claims.email,
        is_admin: claims.is_admin,
    };
    request.extensions_mut().insert(user);

    // Proceed to the next handler
    Ok(next.run(request).await)
}
