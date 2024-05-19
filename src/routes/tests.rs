#[cfg(test)]
mod tests {
    use core::panic;
    use std::env;

    use crate::models::user::{CreateUser, User};
    use dotenv::dotenv;
    use log::error;
    use rand::{distributions::Alphanumeric, Rng}; // 0.8
    use reqwest::StatusCode;
    use sqlx::PgPool;
    use uuid::Uuid;
    async fn get_live_db_pool() -> PgPool {
        dotenv().ok();
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        PgPool::connect(&database_url)
            .await
            .expect("Failed to connect to the live database")
    }
    #[tokio::test]
    async fn test_create_user() {
        let pool = get_live_db_pool().await;
        let client = reqwest::Client::new();
        let s: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(7)
            .map(char::from)
            .collect();
        let payload = CreateUser {
            username: s.to_string(),
        };
        let response = client
            .post("http://localhost:3000/user")
            .json(&payload)
            .send()
            .await;

        if let Err(e) = response {
            panic!("Failed to create user: {}", e);
        }
        let response = response.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let json: serde_json::Value = match response.json().await {
            Ok(json) => json,
            Err(e) => {
                panic!("Failed to parse response: {}", e);
            }
        };
        let api_key = match json["api_key"].as_str() {
            Some(key) => key,
            None => {
                panic!("Missing api_key");
            }
        };
        println!("API key: {}", api_key);
        let user = match sqlx::query_as!(User, "SELECT * FROM \"user\" WHERE api_key = $1", api_key)
            .fetch_one(&pool)
            .await
        {
            Ok(user) => user,
            Err(e) => {
                println!("Failed to find user: {}", e);
                panic!("{e}");
            }
        };

        assert_eq!(user.username, payload.username);

        // Delete the user after the test
        match sqlx::query!("DELETE FROM \"user\" WHERE api_key = $1", api_key)
            .execute(&pool)
            .await
        {
            Ok(_) => println!("User deleted"),
            Err(e) => panic!("Failed to delete user: {}", e),
        }
    }

    #[tokio::test]
    async fn test_get_user() {
        let pool = get_live_db_pool().await;
        let client = reqwest::Client::new();
        //let user_id = Uuid::new_v4();
        let api_key = Uuid::new_v4().to_string();
        let s: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(7)
            .map(char::from)
            .collect();
        match sqlx::query!(
            r#"
            insert into "user"(api_key, username)
            values ($1, $2)
        "#,
            api_key.to_string(),
            s
        )
        .execute(&pool)
        .await
        {
            Ok(_) => println!("Test user inserted"),
            Err(e) => {
                error!("Failed to insert test user: {}", e);
                panic!("{e}");
            }
        }
        let response = client
            .get("http://localhost:3000/user")
            .header("api-key", api_key.clone())
            .send()
            .await;

        if let Err(e) = response {
            panic!("Failed to get user: {}", e);
        }
        let response = response.unwrap();
        let status = response.status().clone();
        let resp: String = match response.text().await {
            Ok(r) => r,
            Err(e) => {
                println!("Failed to parse response: {}", e);
                panic!("{e}");
            }
        };
        println!("{resp}");
        assert_eq!(status, StatusCode::OK);

        // Delete the user after the test
        match sqlx::query!("DELETE FROM \"user\" WHERE api_key = $1", api_key)
            .execute(&pool)
            .await
        {
            Ok(_) => println!("User deleted"),
            Err(e) => panic!("Failed to delete user: {}", e),
        }
    }
}

