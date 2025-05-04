use dotenvy::dotenv;
use sea_orm::{Database, DatabaseConnection, DbErr};
use std::env;

pub struct Configuration {
    pub database_url: String,
    pub db_max_connection: u32,
    pub jwt_secret: String,
    pub jwt_expiration: i64,
    pub server_port: String,
    pub magic_slide_access_id: String,
    pub magic_slide_base_url: String,
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
            jwt_secret: env::var("JWT_SECRET").expect(" JWT_SECRET must be set"),
            jwt_expiration: env::var("JWT_EXPIRATION")
                .unwrap_or_else(|_| "86400".to_string()) // 24 hours
                .parse()
                .expect("JWT_EXPIRATION must be a number"),
            server_port: env::var("SERVER_PORT").unwrap_or_else(|_| "8080".to_string()),
            magic_slide_access_id: env::var("Magic_Slide_Access_ID").unwrap(),
            magic_slide_base_url: env::var("Magic_Slide_Base_URL").unwrap(),
        }
    }

    pub async fn establish_connection(&self) -> Result<DatabaseConnection, DbErr> {
        let db_options = sea_orm::ConnectOptions::new(&self.database_url)
            .max_connections(self.db_max_connection)
            .min_connections(5)
            .connect_timeout(std::time::Duration::from_secs(3600))
            .idle_timeout(std::time::Duration::from_secs(3600))
            .sqlx_logging(true)
            .to_owned();

        let conn = Database::connect(db_options).await?;

        Ok(conn)
    }
}
