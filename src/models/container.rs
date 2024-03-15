use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize)]
pub struct Container {
    pub container_id: Uuid,
    pub language: String,
    pub port: i32,
    pub user_id: Option<Uuid>,
}
#[derive(Deserialize)]
pub struct NewContainer {
    pub language: String,
}
#[derive(Deserialize)]
pub struct QueryContainer {
    pub container_id: String,
}
#[derive(Deserialize, Serialize)]
pub struct ReturnMessage {
    pub container_id: String,
    pub message: String,
}
