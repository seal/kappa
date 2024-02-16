use axum::http::StatusCode;
use axum::Json;
use axum::{
    extract::Request,
    routing::{get, post},
    Router,
};
use errors::error::AppError;
use models::user::User;
use std::task::{Context, Poll};
use tower::{Layer, Service};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use std::time::Duration;

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

    let state = MyState {
        db_pool: pool.clone(),
        user: None,
    };
    let app = Router::new()
        .route("/error", get(routes::health::error_handler))
        .route("/user", post(routes::user::create_user))
        .route("/user", get(routes::user::get_user))
        .route("/containers", get(routes::containers::get_containers))
        .route_layer(MyLayer { state })
        .route("/containers", post(routes::containers::new_container))
        .with_state(pool);
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    info!("Starting listener on port 3000");
    axum::serve(listener, app).await.unwrap();
}

#[derive(Debug, Clone)]
struct MyState {
    db_pool: sqlx::PgPool,
    user: Option<User>,
}

#[derive(Clone)]
struct MyLayer {
    state: MyState,
}

impl<S> Layer<S> for MyLayer {
    type Service = MyService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        MyService {
            inner,
            state: self.state.clone(),
        }
    }
}

#[derive(Clone)]
struct MyService<S> {
    inner: S,
    state: MyState,
}

impl<S, B> Service<Request<B>> for MyService<S>
where
    S: Service<Request<B>>,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = S::Future;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<B>) -> Self::Future {
        let state = self.state.clone();
        let db_future = async move {
            let user: Result<User, sqlx::Error> = sqlx::query_as!(
                User,
                r#"
                SELECT * FROM "user"
                WHERE api_key = $1
                "#,
                "92fd244a-df5a-4025-a8c9-c77af829c164"
            )
            .fetch_one(&state.db_pool)
            .await;

            // Handle the result of the database call here

            if let Ok(user) = user {
                println!("{:?}", user);
            } else {
                print!("none");
            }
        };
        tokio::spawn(db_future);
        self.inner.call(req) // Continue processing the request
    }
}
