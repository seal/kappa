use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use sqlx::postgres::PgPool;

use crate::models;

pub async fn auth(
    req: Request,
    //db_pool: State<PgPool>,
    executor: tokio::runtime::Handle,

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

    let result = sqlx::query_as!(
        models::user::User,
        r#"SELECT user_id, api_key, username FROM "user" WHERE api_key = $1"#,
        auth_header.to_string()
    )
    .fetch_one(executor)
    .await;

    match result {
        Ok(user) => {
            req.extensions_mut().insert(user);
            Ok(next.run(req).await)
        }
        Err(_) => Err(StatusCode::UNAUTHORIZED),
    }
}
