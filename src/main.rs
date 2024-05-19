use axum::middleware;
use tracing_subscriber::Registry;

use axum::routing::delete;
use axum::{
    routing::{get, post},
    Router,
};
use routes::middleware::{api_key_auth, trigger_auth};
use std::time::Duration;
use tracing_subscriber::layer::SubscriberExt;

use std::{fs::File, sync::Arc};
use tracing::info;
use tracing_subscriber::{filter, prelude::*};

use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

use dotenv::dotenv;
mod docker;
mod errors;
mod models;
mod routes;
mod utils;

async fn create_app(pool: PgPool) -> Router {
    let protected = Router::new()
        .route("/user", get(routes::user::get_user))
        .route("/containers", post(routes::containers::new_container))
        .route("/containers", get(routes::containers::get_containers))
        .route("/containers", delete(routes::containers::delete_container))
        .layer(middleware::from_fn_with_state(pool.clone(), api_key_auth));
    let trigger = Router::new()
        .route(
            "/trigger",
            get(routes::containers::trigger_container).post(routes::containers::trigger_container),
        )
        .layer(middleware::from_fn_with_state(pool.clone(), trigger_auth));
    Router::new()
        .route("/user", post(routes::user::create_user))
        .route("/error", get(routes::health::error_handler))
        .merge(protected)
        .merge(trigger)
        .with_state(pool)
}

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

    let metrics_layer = /* ... */ filter::LevelFilter::DEBUG;

    let subscriber = Registry::default()
        .with(
            stdout_log
                .with_filter(filter::LevelFilter::DEBUG)
                .and_then(debug_log)
                .with_filter(filter::filter_fn(|metadata| {
                    !metadata.target().starts_with("metrics")
                })),
        )
        .with(metrics_layer.with_filter(filter::filter_fn(|metadata| {
            metadata.target().starts_with("metrics")
        })));

    tracing::subscriber::set_global_default(subscriber)
        .expect("Failed to set global default subscriber");

    let app = create_app(pool).await;
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    info!("Starting listener on port 3000");
    axum::serve(listener, app).await.unwrap();
}
