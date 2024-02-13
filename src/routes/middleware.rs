use axum::{
    extract::{Extension, Request, State},
    http::StatusCode,
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use sqlx::postgres::PgPool;

use crate::models;

pub async fn auth(
    mut req: Request,
    db_pool: State<PgPool>,
    next: Next,
) -> Result<Response, StatusCode> {
    let auth_header = req
        .headers()
        .get(http::header::AUTHORIZATION)
        .and_then(|header| header.to_str().ok());

    let auth_header = if let Some(auth_header) = auth_header {
        auth_header
    } else {
        return Err(StatusCode::UNAUTHORIZED);
    };

    if let Some(current_user) = authorize_current_user(auth_header, db_pool).await {
        req.extensions_mut().insert(current_user);
        Ok(next.run(req).await)
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

async fn authorize_current_user(
    _auth_token: &str,
    db_pool: State<PgPool>,
) -> Option<models::user::User> {
    // Use the db_pool to interact with the database
    // For example, perform a SQL query using sqlx
    // Replace the following with your actual query logic
    let result = sqlx::query_as!(
        models::user::User,
        r#"SELECT user_id, api_key, username FROM "user" WHERE api_key = $1"#,
        _auth_token.to_string()
    )
    .fetch_one(db_pool)
    .await;

    match result {
        Ok(user) => Some(user),
        Err(_) => None,
    }
}
