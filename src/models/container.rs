use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Clone)]
pub struct Container {
    pub container_id: Uuid,
    pub language: String,
    pub user_id: Option<Uuid>,
    pub port: Option<i32>,
    pub name: String,
}
#[derive(Deserialize)]
pub struct NewContainer {
    pub language: String,
    pub name: String,
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
