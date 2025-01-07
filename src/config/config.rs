use dotenvy::dotenv;
use sea_orm::{Database, DatabaseConnection, DbErr};
use std::env;

pub struct Configuration {
    pub database_url: String,
    pub db_max_connection: u32,
    pub jwt_secret: String,
    pub jwt_expiration: i64,
}

impl Configuration {
    pub fn init_env() -> Self {
        dotenv().ok();

        Self {
            database_url: env::var("DATABASE_URL").expect("DATABASE_URL must be set"),
            db_max_connection: env::var("DB_MAX_CONNECTIONS")
                .unwrap_or_else(|_| "10".to_string())
                .parse()
                .expect("DB_MAX_CONNECTIONS must be set"),
            jwt_secret: env::var("JWT_SECRET").expect("JWT_SECRET must be set"),
            jwt_expiration: env::var("JWT_EXPIRATION")
                .unwrap_or_else(|_| "86400".to_string()) // 24 hours
                .parse()
                .expect("JWT_EXPIRATION must be a number"),
        }
    }

    pub async fn establish_connection(&self) -> Result<DatabaseConnection, DbErr> {
        let conn = Database::connect(&self.database_url).await?;

        //Optional: Run migrations here
        Ok(conn)
    }
}
