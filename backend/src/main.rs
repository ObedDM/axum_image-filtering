use tokio::{net::TcpListener, signal::unix::{signal, SignalKind}, sync::oneshot};

use axum::{extract::{Path, State}, http::StatusCode, routing::{get, patch, post}, Json, Router};

use serde::{Deserialize, Serialize};
use serde_json::json;

use reqwest::{Client, multipart};

use dotenvy;

#[tokio::main]
async fn main() {
    // Open our .env file and extract server address
    dotenvy::dotenv().expect("Cannot access .env file");
    let server_address = std::env::var("SERVER_ADDRESS").unwrap_or("127.0.0.1:3000".to_string());

    // Shutdown setup
    let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();
    tokio::spawn(async move {
        let mut sigint = signal(SignalKind::interrupt()).unwrap();
        sigint.recv().await;
        let _ = shutdown_tx.send(());
    });

    // Create TCPlistener
    let listener = TcpListener::bind(server_address).await.expect("Could not create tcp listener");
    println!("listening on: {}", listener.local_addr().unwrap());

    // Assign routes
    let app = Router::new()
            .route("/", get(|| async { "Hello World" } ))
            .route("/hi", post(post_img));

    // Serve the app and handle shutdown
    tokio::select! {
        _ =  axum::serve(listener, app) => {},
        _ = shutdown_rx => { println!("\nShutting down...") },
    }
}

async fn post_img() -> Result<(StatusCode, String), (StatusCode, String)> {
    todo!()
}