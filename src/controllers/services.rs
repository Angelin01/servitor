use crate::errors::ServiceError;
use crate::middleware::auth::auth_middleware;
use crate::models::services::{ServiceResponse, ServiceStatusResponse};
use crate::state::AppState;
use crate::systemd::SystemdUnitProxy;
use axum::{
	Json, Router,
	extract::Path,
	extract::State,
	middleware,
	response::Result,
	routing::{get, post},
};
use chrono::DateTime;
use log::{error, info};

pub fn create_router(state: AppState) -> Router<AppState> {
	let mut router = Router::new()
		.route("/{service}/start", post(start_service))
		.route("/{service}/stop", post(stop_service))
		.route("/{service}/restart", post(restart_service))
		.route("/{service}/reload", post(reload_service))
		.route("/{service}/status", get(status_service));

	if state.has_auth() {
		info!("Authentication is enabled");
		router = router.layer(middleware::from_fn_with_state(
			state.clone(),
			auth_middleware,
		));
	} else {
		info!("Running without authentication");
	}

	router
}

async fn start_service(
	State(state): State<AppState>,
	Path(service): Path<String>,
) -> Result<Json<ServiceResponse>, ServiceError> {
	ensure_service_allowed(&state, &service)?;
	info!("Starting service: {service}");

	state.manager_proxy.start_unit(&service, "replace").await.map_err(|e| {
		error!("Failed to start service {service}: {e}");
		e
	})?;

	Ok(Json(ServiceResponse {
		service,
		status: "starting".into(),
	}))
}

async fn stop_service(
	State(state): State<AppState>,
	Path(service): Path<String>,
) -> Result<Json<ServiceResponse>, ServiceError> {
	ensure_service_allowed(&state, &service)?;
	info!("Stopping service: {service}");

	state.manager_proxy.stop_unit(&service, "replace").await.map_err(|e| {
		error!("Failed to stop service {service}: {e}");
		e
	})?;

	Ok(Json(ServiceResponse {
		service,
		status: "stopping".into(),
	}))
}

async fn restart_service(
	State(state): State<AppState>,
	Path(service): Path<String>,
) -> Result<Json<ServiceResponse>, ServiceError> {
	ensure_service_allowed(&state, &service)?;
	info!("Restarting service: {service}");

	state.manager_proxy.restart_unit(&service, "replace").await.map_err(|e| {
		error!("Failed to restart service {service}: {e}");
		e
	})?;

	Ok(Json(ServiceResponse {
		service,
		status: "restarting".into(),
	}))
}

async fn reload_service(
	State(state): State<AppState>,
	Path(service): Path<String>,
) -> Result<Json<ServiceResponse>, ServiceError> {
	ensure_service_allowed(&state, &service)?;
	info!("Reloading service: {service}");

	state.manager_proxy.reload_unit(&service, "replace").await.map_err(|e| {
		error!("Failed to reload service {service}: {e}");
		e
	})?;

	Ok(Json(ServiceResponse {
		service,
		status: "reloading".into(),
	}))
}

async fn status_service(
	State(state): State<AppState>,
	Path(service): Path<String>,
) -> Result<Json<ServiceStatusResponse>, ServiceError> {
	ensure_service_allowed(&state, &service)?;
	info!("Fetching status for service: {service}");

	let unit_path = state.manager_proxy.get_unit(&service).await.map_err(|e| {
		error!("Failed to retrieve unit path for {service}: {e}");
		e
	})?;

	let unit_proxy = SystemdUnitProxy::new(&state.dbus_conn, unit_path.to_string()).await.map_err(|e| {
		error!("Failed to retrieve systemd unit for {service}: {e}");
		e
	})?;

	let state = unit_proxy.active_state().await.map_err(|e| {
		error!("Failed to get active state for {service}: {e}");
		e
	})?;

	let sub_state = unit_proxy.sub_state().await.map_err(|e| {
		error!("Failed to get sub-state for {service}: {e}");
		e
	})?;

	let since = unit_proxy
		.state_change_timestamp()
		.await
		.ok()
		.and_then(|t| DateTime::from_timestamp_micros(t as i64));

	info!(
		"Service {service}: state={state}, sub_state={sub_state}, since={since:?}",
	);

	Ok(Json(ServiceStatusResponse {
		service,
		state,
		sub_state,
		since,
	}))
}

fn ensure_service_allowed(state: &AppState, service: &str) -> Result<(), ServiceError> {
	if service.is_empty() {
		info!("Access denied to an empty service name");
		return Err(ServiceError::InvalidUnit);
	}

	match &state.allowed_services {
		None => Ok(()),
		Some(a) => {
			if a.contains(service) {
				Ok(())
			} else {
				info!("Access denied to service outside allowlist: {service}");
				Err(ServiceError::InvalidUnit)
			}
		}
	}
}
