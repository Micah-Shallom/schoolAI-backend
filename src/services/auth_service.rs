use bcrypt::{hash, DEFAULT_COST};
use chrono::Utc;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use uuid::Uuid;
use validator::Validate;

use crate::config::jwt::JwtConfig;
use crate::models::user::{ActiveModel as UserActiveModel, AuthResponse, Column::Email, Entity as User, RegisterRequest};
use crate::utils::errors::AppError;

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
