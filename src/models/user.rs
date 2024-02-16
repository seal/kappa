use serde::{Deserialize, Serialize};
use uuid::Uuid;
#[derive(Deserialize, Debug)]
pub struct CreateUser {
    pub username: String,
}

#[derive(Serialize, Clone, Debug)]
pub struct User {
    pub user_id: Uuid,
    pub username: String,
    pub api_key: String,
}
