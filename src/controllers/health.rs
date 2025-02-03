use crate::models::health::HealthResponse;
use crate::state::AppState;
use axum::routing::get;
use axum::{Json, Router};

pub fn create_router<'a>() -> Router<AppState> {
	Router::new().route("/health", get(health_check))
}

async fn health_check() -> Json<HealthResponse> {
	Json(HealthResponse {
		status: "OK".into(),
	})
}
