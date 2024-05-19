#[cfg(test)]
mod tests {
    use crate::create_app;
    use axum::http::StatusCode;
    use axum_test::multipart::MultipartForm;
    use axum_test::TestServer;
    use http::{HeaderName, HeaderValue};
    use rand::{distributions::Alphanumeric, Rng};
    use serde_json::{json, Value};
    use sqlx::PgPool;
    use uuid::Uuid;

    #[sqlx::test]
    async fn test_create_user(pool: PgPool) {
        let app = create_app(pool.clone()).await;
        let server = TestServer::new(app).expect("Failed to create test server");

        let s: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(7)
            .map(char::from)
            .collect();
        let payload = json!({
            "username": s,
        });

        let response = server
            .post("/user")
            .content_type(&"application/json")
            .json(&payload)
            .await;

        assert_eq!(response.status_code(), StatusCode::OK);
        let json: Value = serde_json::from_str(&response.text()).unwrap();
        let api_key = match json["api_key"].as_str() {
            Some(key) => key,
            None => {
                panic!("Missing api_key");
            }
        };

        let user = sqlx::query!("SELECT * FROM \"user\" WHERE api_key = $1", api_key)
            .fetch_one(&pool)
            .await
            .expect("Failed to find user");

        assert_eq!(user.username, payload["username"].as_str().unwrap());
    }

    #[sqlx::test]
    async fn test_get_user(pool: PgPool) {
        let app = create_app(pool.clone()).await;
        let server = TestServer::new(app).expect("Failed to create test server");

        let api_key = Uuid::new_v4().to_string();
        let s: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(7)
            .map(char::from)
            .collect();

        sqlx::query!(
            r#"
            INSERT INTO "user"(api_key, username)
            VALUES ($1, $2)
        "#,
            api_key,
            s
        )
        .execute(&pool)
        .await
        .expect("Failed to insert test user");
        let api_key_static: &'static str = Box::leak(api_key.clone().into_boxed_str());
        let response = server
            .get("/user")
            .add_header(
                HeaderName::from_static("api-key"),
                HeaderValue::from_static(api_key_static),
            )
            .await;

        assert_eq!(response.status_code(), StatusCode::OK);
    }
    #[sqlx::test]
    async fn test_delete_container(pool: PgPool) {
        // Create a test user and container
        let user_id = Uuid::new_v4();
        let api_key = Uuid::new_v4().to_string();
        let container_id = Uuid::new_v4();
        sqlx::query!(
            r#"
        INSERT INTO "user"(user_id, username,api_key)
        VALUES ($1, $2, $3)
        "#,
            user_id,
            "ahh",
            api_key
        )
        .execute(&pool)
        .await
        .expect("Failed to insert test user");
        sqlx::query!(
            r#"
        INSERT INTO "container"(container_id, language, user_id, port, name)
        VALUES ($1, 'go', $2, 8080, 'Test Container')
        "#,
            container_id,
            user_id
        )
        .execute(&pool)
        .await
        .expect("Failed to insert test container");

        let app = create_app(pool.clone()).await;
        let server = TestServer::new(app).expect("Failed to create test server");
        let response = server
            .delete("/containers")
            .add_query_param("container_id", container_id.to_string())
            .add_header(
                HeaderName::from_static("api-key"),
                HeaderValue::from_str(api_key.as_str()).expect("Api key from str died"),
            )
            .await;
        assert_eq!(response.status_code(), StatusCode::OK);
        let json: Value = serde_json::from_str(&response.text()).unwrap();
        assert_eq!(json["message"], "Successfully deleted container");
    }

    #[sqlx::test]
    async fn test_trigger_container(pool: PgPool) {
        // Create a test user and container
        let user_id = Uuid::new_v4();
        let api_key = Uuid::new_v4().to_string();
        let container_id = Uuid::new_v4();
        sqlx::query!(
            r#"
        INSERT INTO "user"(user_id, username,api_key)
        VALUES ($1, $2, $3)
        "#,
            user_id,
            "random",
            api_key
        )
        .execute(&pool)
        .await
        .expect("Failed to insert test user");
        sqlx::query!(
            r#"
        INSERT INTO "container"(container_id, language, user_id, port, name)
        VALUES ($1, 'go', $2, 8080, 'Test Container')
        "#,
            container_id,
            user_id
        )
        .execute(&pool)
        .await
        .expect("Failed to insert test container");

        let app = create_app(pool.clone()).await;
        let server = TestServer::new(app).expect("Failed to create test server");

        let response = server
            .get("/trigger")
            .add_header(
                HeaderName::from_static("api-key"),
                HeaderValue::from_str(api_key.as_str()).expect("Str api key died"),
            )
            .add_header(
                HeaderName::from_static("container"),
                HeaderValue::from_str(&container_id.to_string()).expect("failed str"),
            )
            .await;

        assert_eq!(response.status_code(), StatusCode::OK);
    }

    #[sqlx::test]
    async fn test_new_container(pool: PgPool) {
        let user_id = Uuid::new_v4();
        let api_key = Uuid::new_v4().to_string();
        sqlx::query!(
            r#"
        INSERT INTO "user"(user_id, username,api_key)
        VALUES ($1, $2, $3)
        "#,
            user_id,
            "ahh",
            api_key
        )
        .execute(&pool)
        .await
        .expect("Failed to insert test user");

        let app = create_app(pool.clone()).await;
        let server = TestServer::new(app).expect("Failed to create test server");

        let form = MultipartForm::new()
            .add_text("language", "go")
            .add_text("name", "Test Container");

        let response = server
            .post("/container")
            .add_query_param("language", "go")
            .add_query_param("name", "test")
            .add_header(
                HeaderName::from_static("api-key"),
                HeaderValue::from_str(api_key.as_str()).expect("failed to str"),
            )
            .multipart(form)
            .await;
        // Should fail, no file
        assert_eq!(response.status_code(), StatusCode::BAD_REQUEST);
    }

    #[sqlx::test]
    async fn test_get_containers(pool: PgPool) {
        let user_id = Uuid::new_v4();
        let api_key = Uuid::new_v4().to_string();
        let container_id = Uuid::new_v4();
        sqlx::query!(
            r#"
        INSERT INTO "user"(user_id, username,api_key)
        VALUES ($1, $2, $3)
        "#,
            user_id,
            "ahh",
            api_key
        )
        .execute(&pool)
        .await
        .expect("Failed to insert test user");
        sqlx::query!(
            r#"
        INSERT INTO "container"(container_id, language, user_id, port, name)
        VALUES ($1, 'go', $2, 8080, 'Test Container')
        "#,
            container_id,
            user_id
        )
        .execute(&pool)
        .await
        .expect("Failed to insert test container");

        let app = create_app(pool.clone()).await;
        let server = TestServer::new(app).expect("Failed to create test server");

        let response = server
            .get("/containers")
            .add_header(
                HeaderName::from_static("api-key"),
                HeaderValue::from_str(api_key.as_str()).expect("Str died"),
            )
            .await;
        assert_eq!(response.status_code(), StatusCode::OK);
        let json: Value = serde_json::from_str(&response.text()).unwrap();
        assert_eq!(json.as_array().unwrap().len(), 1);
    }
}
