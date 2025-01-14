use sea_orm::DatabaseConnection;

use crate::{models::features::AcademicContentRequest, utils::errors::AppError};

pub async fn content_service(
    _db: &DatabaseConnection,
    _req: AcademicContentRequest,
) -> Result<(), AppError> {
    Ok(())
}
