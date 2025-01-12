use crate::{models::blacklist::{Column, Entity as Blacklist}, router::AppState};
use sea_orm::{EntityTrait, QueryFilter, ColumnTrait};
use axum::{
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
};
use uuid::Uuid; // assuming AppState is defined with a jwt_config field

#[derive(Clone, Debug)]
pub struct AuthenticatedUser {
    pub user_id: Uuid,
    pub email: String,
    pub is_admin: bool,
}

pub async fn auth_middleware(
    State(app_state): State<AppState>,
    mut request: Request<axum::body::Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    // Extract jwt_config from the AppState
    let jwt_config = &app_state.jwt_config;

    // Get Authorization header
    let auth_header = request
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;

    // Extract token
    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let is_blacklisted =  Blacklist::find()
        .filter(Column::Token.eq(token))
        .one(&app_state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if is_blacklisted.is_some() {
        return Err(StatusCode::UNAUTHORIZED);
    }

    // Validate token
    let claims = jwt_config
        .validate_token(token)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

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
