use serde::Serialize;

#[derive(Serialize)]
pub struct ServiceResponse {
    pub service: String,
    pub status: String,
}
