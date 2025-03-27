use std::path::Path;
use tokio::{fs::File, io::AsyncWriteExt, net::TcpListener, signal::unix::{signal, SignalKind}, sync::oneshot};

use axum::{extract::{Multipart, Path as AxumPath, State}, http::StatusCode, response::Html, routing::{get, patch, post}, Json, Router};

use serde::{Deserialize, Serialize};
use serde_json::json;

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
            .route("/", get(|| async { "Hello World" }))
            .route("/images", post(post_img));

    // Serve the app and handle shutdown
    tokio::select! {
        _ =  axum::serve(listener, app) => {},
        _ = shutdown_rx => { println!("\nShutting down...") },
    }
}

async fn post_img(mut multipart: Multipart) -> Result<((StatusCode, String)), (StatusCode, String)> {
    // Store image data and name through iterations
    let mut image_name: Option<String> = None;
    let mut image_data: Option<Vec<u8>> = None;
    
    while let Some(field) = multipart.next_field().await.expect("Multipart error") {
        let field_name = field.name().unwrap_or("unnamed").to_string();

        match field_name.as_str() {
            "Image" => {
                // Extract image data
                let data = field.bytes().await.expect("No image data found").to_vec();
                image_data = Some(data);
            },

            "ImageName" => {
                let name = field.text().await.expect("No image name found");
                image_name = Some(name);
            },

            _ => {}
        }
    }
    
    if image_data.is_none() || image_name.is_none() {
        return Err((StatusCode::BAD_REQUEST, "Image data or name not found".to_string()));
    }

    // Write image to path
    let formatted_path = format!("../public/uploads/{}.jpg", &image_name.unwrap());
    let uploads_path = Path::new(&formatted_path);

    let mut file = File::create(uploads_path).await.
            map_err(|e| {
            return (StatusCode::INTERNAL_SERVER_ERROR, format!("Error creating file: {}", e));
            })?;

    file.write_all(&image_data.unwrap()).await
            .map_err(|e| {
            return (StatusCode::INTERNAL_SERVER_ERROR, format!("Error saving image: {}", e));
            })?;

    Ok((StatusCode::CREATED,
        json!( {"message": "success", "id": "1" } ).to_string()
    ))

}