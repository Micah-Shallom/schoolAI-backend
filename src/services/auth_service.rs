use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::{TimeZone, Utc};
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use uuid::Uuid;
use validator::Validate;

use crate::config::jwt::JwtConfig;
use crate::models::blacklist::{ActiveModel as BlacklistActiveModel, Column, Entity as Blacklist};
use crate::models::user::{
    ActiveModel as UserActiveModel, AuthResponse, Column::Email, Entity as User, RegisterRequest,
};
use crate::models::user::{LoginRequest, LogoutRequest};
use crate::utils::errors::AppError;

pub async fn logout_user_service(
    db: &DatabaseConnection,
    jwt_config: &JwtConfig,
    payload: LogoutRequest,
) -> Result<(), AppError> {
    let token = payload.token;

    //check if token has already been blacklisted
    let is_blacklisted = Blacklist::find()
        .filter(Column::Token.eq(token.clone()))
        .one(db)
        .await
        .map_err(|_| AppError::InternalServerError("Database query error".to_string()))?;

    if is_blacklisted.is_some() {
        return Err(AppError::Unauthorized(
            "Token has already been blacklisted".to_string(),
        ));
    }

    let token_data = jwt_config
        .validate_token(&token)
        .map_err(|e| AppError::Unauthorized(e.to_string()))?;

    let expires_at = token_data.exp;

    let blacklist_entry = BlacklistActiveModel {
        token: Set(token),
        expires_at: Set(Utc
            .timestamp_opt(expires_at as i64, 0)
            .single()
            .ok_or_else(|| AppError::InternalServerError("invalid timestamp".to_string()))?),
    };

    blacklist_entry
        .insert(db)
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    Ok(())
}

pub async fn login_user_service(
    db: &DatabaseConnection,
    jwt_config: &JwtConfig,
    payload: LoginRequest,
) -> Result<AuthResponse, AppError> {
    payload
        .validate()
        .map_err(|e| AppError::BadRequest(e.to_string()))?;

    let user = User::find()
        .filter(Email.eq(&payload.email))
        .one(db)
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?
        .ok_or_else(|| AppError::Unauthorized("Invalid email or password".to_string()))?;

    verify(&payload.password, &user.password_hash)
        .map_err(|_| AppError::Unauthorized("Invalid email or password".to_string()))?
        .then(|| ())
        .ok_or_else(|| AppError::Unauthorized("Invalid email or password".to_string()))?;

    let token = jwt_config
        .generate_token(user.id, user.email.clone(), user.is_admin)
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    Ok(AuthResponse {
        token,
        user_id: user.id,
        email: user.email,
        first_name: user.first_name.unwrap_or_default(),
        last_name: user.last_name.unwrap_or_default(),
    })
}

pub async fn register_user_service(
    db: &DatabaseConnection,
    jwt_config: &JwtConfig,
    payload: RegisterRequest,
) -> Result<AuthResponse, AppError> {
    payload
        .validate()
        .map_err(|e| AppError::BadRequest(e.to_string()))?;

    //check if email is already registered
    let existing_user = User::find()
        .filter(Email.eq(&payload.email))
        .one(db)
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    if let Some(_) = existing_user {
        return Err(AppError::Conflict(
            "Email is already registered".to_string(),
        ));
    }

    let password_hash = hash(&payload.password, DEFAULT_COST)
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    // Create a new user
    let user = UserActiveModel {
        id: Set(Uuid::new_v4()),
        email: Set(payload.email.clone()),
        password_hash: Set(password_hash),
        first_name: Set(Some(payload.first_name.clone())),
        last_name: Set(Some(payload.last_name.clone())),
        created_at: Set(Utc::now()),
        updated_at: Set(Utc::now()),
        is_admin: Set(false),
    };
    let user = user
        .insert(db)
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    // Generate a JWT token
    let token = jwt_config
        .generate_token(user.id, user.email.clone(), false)
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    // Return the response
    Ok(AuthResponse {
        token,
        user_id: user.id,
        email: user.email,
        first_name: payload.first_name.clone(),
        last_name: payload.last_name.clone(),
    })
}
