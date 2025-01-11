mod config;
mod controllers;
mod middleware;
mod models;
mod router;
mod services;
mod utils;

use config::jwt::JwtConfig;
use migration::{Migrator, MigratorTrait};

use crate::config::config::Configuration;

#[tokio::main]
async fn main() {
    let configuration = Configuration::init_env();

    //Run migrations before establishing connection
    println!("Running database migrations...");
    let db_url = &configuration.database_url;

    let migrations_connection = sea_orm::Database::connect(db_url)
        .await
        .expect("Failed to connect to database for migrations");

    Migrator::up(&migrations_connection, None).await.unwrap();

    //establish db connection
    let db = match configuration.establish_connection().await {
        Ok(conn) => {
            println!("Database connection established successfully");
            conn
        }
        Err(e) => {
            eprintln!("Failed to establish database connection: {}", e);
            return;
        }
    };

    //create jwt config
    let jwt_config = JwtConfig::new(
        configuration.jwt_secret.clone(),
        configuration.jwt_expiration,
    );

    let app = router::create_router(db, jwt_config); //share db connection with all handlers

    let port = configuration.server_port;
    let addr = format!("0.0.0.0:{}", port);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();

    println!("Server listening on http://{}", addr);

    axum::serve(listener, app).await.unwrap();
}
