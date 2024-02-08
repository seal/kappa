use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize)]
pub struct Container {
    pub container_id: Uuid,
    pub language: String,
    pub port: i32,
}
#[derive(Deserialize)]
pub struct NewContainer {
    pub language: String,
    pub port: i32,
}
