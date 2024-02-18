use axum::middleware;

use axum::{
    routing::{get, post},
    Router,
};
use routes::middleware::api_key_auth;
use std::time::Duration;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use std::{fs::File, sync::Arc};
use tracing::info;
use tracing_subscriber::{filter, prelude::*};

use sqlx::postgres::PgPoolOptions;

use dotenv::dotenv;
mod errors;
mod models;
mod routes;
#[tokio::main]
async fn main() {
    let stdout_log = tracing_subscriber::fmt::layer().pretty();
    dotenv().ok();
    let file = File::create("debug.log");
    let file = match file {
        Ok(file) => file,
        Err(error) => panic!("Error: {:?}", error),
    };
    let debug_log = tracing_subscriber::fmt::layer().with_writer(Arc::new(file));
    let db_connection_str = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:password@localhost".to_string());
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(3))
        .connect(&db_connection_str)
        .await
        .expect("can't connect to database");

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
    let protected = Router::new()
        .route("/user", get(routes::user::get_user))
        .route("/containers", post(routes::containers::new_container))
        .route("/containers", get(routes::containers::get_containers))
        .layer(middleware::from_fn_with_state(pool.clone(), api_key_auth));
    let app = Router::new()
        .route("/user", post(routes::user::create_user))
        .route("/error", get(routes::health::error_handler))
        .merge(protected)
        .with_state(pool);
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    info!("Starting listener on port 3000");
    axum::serve(listener, app).await.unwrap();
}
