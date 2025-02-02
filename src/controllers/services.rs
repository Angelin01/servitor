use crate::errors::ServiceError;
use crate::models::services::{ServiceResponse, ServiceStatusResponse};
use crate::state::AppState;
use crate::systemd::SystemdUnitProxy;
use axum::{Json, Router, extract::Path, extract::State, response::Result, routing::{get, post}, middleware};
use chrono::DateTime;
use crate::middleware::auth::auth_middleware;

pub fn create_router(state: AppState) -> Router<AppState> {
	let mut router = Router::new()
		.route("/{service}/start", post(start_service))
		.route("/{service}/stop", post(stop_service))
		.route("/{service}/restart", post(restart_service))
		.route("/{service}/reload", post(reload_service))
		.route("/{service}/status", get(status_service));
	if state.config.auth_token.is_some() {
		router = router.layer(middleware::from_fn_with_state(state.clone(), auth_middleware));
	}

	router
}

async fn start_service(
	State(state): State<AppState>,
	Path(service): Path<String>,
) -> Result<Json<ServiceResponse>, ServiceError> {
	state.manager_proxy.start_unit(&service, "replace").await?;
	Ok(Json(ServiceResponse {
		service,
		status: "starting".into(),
	}))
}

async fn stop_service(
	State(state): State<AppState>,
	Path(service): Path<String>,
) -> Result<Json<ServiceResponse>, ServiceError> {
	state.manager_proxy.stop_unit(&service, "replace").await?;
	Ok(Json(ServiceResponse {
		service,
		status: "stopping".into(),
	}))
}

async fn restart_service(
	State(state): State<AppState>,
	Path(service): Path<String>,
) -> Result<Json<ServiceResponse>, ServiceError> {
	state
		.manager_proxy
		.restart_unit(&service, "replace")
		.await?;
	Ok(Json(ServiceResponse {
		service,
		status: "restarting".into(),
	}))
}

async fn reload_service(
	State(state): State<AppState>,
	Path(service): Path<String>,
) -> Result<Json<ServiceResponse>, ServiceError> {
	state
		.manager_proxy
		.reload_unit(&service, "replace")
		.await?;
	Ok(Json(ServiceResponse {
		service,
		status: "restarting".into(),
	}))
}

async fn status_service(
	State(state): State<AppState>,
	Path(service): Path<String>,
) -> Result<Json<ServiceStatusResponse>, ServiceError> {
	let unit_path = state.manager_proxy.get_unit(&service.as_str()).await?;
	let unit_proxy = SystemdUnitProxy::new(&state.dbus_conn, unit_path.to_string()).await?;

	let state = unit_proxy.active_state().await?;
	let sub_state = unit_proxy.sub_state().await?;

	let since = unit_proxy
		.state_change_timestamp()
		.await
		.ok()
		.and_then(|t| DateTime::from_timestamp_micros(t as i64));

	Ok(Json(ServiceStatusResponse {
		service,
		state,
		sub_state,
		since,
	}))
}
