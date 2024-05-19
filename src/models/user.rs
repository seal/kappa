use serde::{Deserialize, Serialize};
use uuid::Uuid;
#[derive(Deserialize, Serialize, Debug)]
pub struct CreateUser {
    pub username: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct User {
    pub user_id: Uuid,
    pub username: String,
    pub api_key: String,
}
