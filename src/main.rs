mod config;

use std::net::SocketAddr;

use axum::{serve::Serve, Router};

use crate::config::config::Configuration;

#[tokio::main]
async fn main() {
    let configuration = Configuration::init_env();

    //establish db connection
    let db = match configuration.establish_connection().await {
        Ok(conn) => {
            println!("Database connection established successfully");
            conn
        },
        Err(e) => {
            eprintln!("Failed to establish database connection: {}", e);
            return;
        }
    };


    let app = Router::new()
    .with_state(db); //share db connection with all handlers


    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    
    axum::serve(listener, app).await.unwrap();
}
