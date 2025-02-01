use crate::models::services::ServiceResponse;
use crate::state::AppState;
use crate::systemd::SystemdUnitProxy;
use axum::{extract::Path, extract::State, routing::{get, post}, Json, Router};

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

async fn status_service(State(state): State<AppState>, Path(service): Path<String>) -> Json<ServiceResponse> {
    let unit_path = state.manager_proxy.get_unit(&service.as_str()).await.unwrap();
    let unit_proxy = SystemdUnitProxy::new(&state.dbus_conn, unit_path.to_string()).await.unwrap();
    let status = unit_proxy.active_state().await.unwrap();
    Json(ServiceResponse { service: unit_path.to_string(), status })
}
