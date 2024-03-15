use serde::Serialize;

#[derive(Serialize)]
pub struct Success {
    pub message: String,
}
