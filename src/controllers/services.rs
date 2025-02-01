use crate::models::services::{ServiceResponse, ServiceStatusResponse};
use crate::state::AppState;
use crate::systemd::SystemdUnitProxy;
use axum::{extract::Path, extract::State, routing::{get, post}, Json, Router};
use chrono::DateTime;

pub fn create_router() -> Router<AppState> {
    Router::new()
        .route("/{service}/start", post(start_service))
        .route("/{service}/stop", post(stop_service))
        .route("/{service}/restart", post(restart_service))
        .route("/{service}/status", get(status_service))
}

async fn start_service(State(state): State<AppState>, Path(service): Path<String>) -> Json<ServiceResponse> {
    state.manager_proxy.start_unit(&service, "replace").await.unwrap();
    Json(ServiceResponse { service, status: "starting".into() })
}

async fn stop_service(State(state): State<AppState>, Path(service): Path<String>) -> Json<ServiceResponse> {
    state.manager_proxy.stop_unit(&service, "replace").await.unwrap();
    Json(ServiceResponse { service, status: "stopping".into() })
}

async fn restart_service(State(state): State<AppState>, Path(service): Path<String>) -> Json<ServiceResponse> {
    state.manager_proxy.restart_unit(&service, "replace").await.unwrap();
    Json(ServiceResponse { service, status: "restarting".into() })
}

async fn status_service(State(state): State<AppState>, Path(service): Path<String>) -> Json<ServiceStatusResponse> {
    let unit_path = state.manager_proxy.get_unit(&service.as_str()).await.unwrap();
    let unit_proxy = SystemdUnitProxy::new(&state.dbus_conn, unit_path.to_string()).await.unwrap();

    let state = unit_proxy.active_state().await.unwrap();
    let sub_state = unit_proxy.sub_state().await.unwrap();

    let since = unit_proxy.state_change_timestamp()
        .await
        .ok()
        .and_then(|t| DateTime::from_timestamp_micros(t as i64))
        .map(|t| t.to_rfc3339());

    Json(ServiceStatusResponse {
        service,
        state,
        sub_state,
        since,
    })
}
