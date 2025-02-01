use serde::Serialize;

#[derive(Serialize)]
pub struct ServiceResponse {
    pub service: String,
    pub status: String,
}

#[derive(Serialize)]
pub struct ServiceStatusResponse {
    pub service: String,
    pub state: String,
    pub sub_state: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub since: Option<String>,
}
