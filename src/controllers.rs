use crate::state::AppState;
use axum::Router;

mod health;
mod services;

pub fn create_router() -> Router<AppState> {
    Router::new()
        .nest("/api/v1/services", services::create_router())
        .merge(health::create_router())
}
