use axum::{
    routing::{get, post},
    Router,
};
use std::{fs::File, sync::Arc};
use tracing::info;
use tracing_subscriber::{filter, prelude::*};
mod errors;
mod models;
mod routes;
#[tokio::main]
async fn main() {
    let stdout_log = tracing_subscriber::fmt::layer().pretty();

    let file = File::create("debug.log");
    let file = match file {
        Ok(file) => file,
        Err(error) => panic!("Error: {:?}", error),
    };
    let debug_log = tracing_subscriber::fmt::layer().with_writer(Arc::new(file));

    // A layer that collects metrics using specific events.
    let metrics_layer = /* ... */ filter::LevelFilter::INFO;

    tracing_subscriber::registry()
        .with(
            stdout_log
                .with_filter(filter::LevelFilter::INFO)
                .and_then(debug_log)
                .with_filter(filter::filter_fn(|metadata| {
                    !metadata.target().starts_with("metrics")
                })),
        )
        .with(metrics_layer.with_filter(filter::filter_fn(|metadata| {
            metadata.target().starts_with("metrics")
        })))
        .init();

    let app = Router::new()
        .route("/error", get(routes::health::error_handler))
        .route("/users", post(routes::health::create_user));
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    info!("Starting listener on port 3000");
    axum::serve(listener, app).await.unwrap();
}
