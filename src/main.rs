use crate::config::Config;
use crate::middleware::auth;
use anyhow::Result;
use log::{error, info};
use state::AppState;
use std::sync::Arc;
use systemd::SystemdManagerProxy;
use tokio::net::TcpListener;
use zbus::Connection;

mod config;
mod controllers;
mod errors;
mod middleware;
mod models;
mod state;
mod systemd;

#[tokio::main]
async fn main() -> Result<()> {
	tracing_subscriber::fmt::init();

	info!("Starting Servitor...");

	let config = Config::from_envs().map_err(|e| {
		error!("Failed to load configuration: {e}");
		e
	})?;

	let dbus_conn = Connection::session().await.map_err(|e| {
		error!("Could not establish a D-Bus session connection: {e}");
		e
	})?;

	let manager_proxy = SystemdManagerProxy::new(&dbus_conn).await.map_err(|e| {
		error!("Failed to initialize Systemd manager proxy: {e}");
		e
	})?;

	let listener = TcpListener::bind(config.bind_address.as_str())
		.await
		.map_err(|e| {
			error!("Failed to bind to {}: {e}", config.bind_address);
			e
		})?;

	let password_hash = auth::read_password_hash(config.auth_token.as_deref())
		.map_err(|e| {
			error!("Failed to read password hash: {e}");
			e
		})?
		.map(Arc::new);

	let state = AppState::new(
		manager_proxy,
		dbus_conn,
		password_hash,
		config.allowlist.clone(),
	);

	match &config.allowlist {
		None => info!("Running without allowlist, all services are reachable"),
		Some(a) => info!(
			"Current allowlist: {}",
			a.iter().cloned().collect::<Vec<_>>().join(", ")
		),
	}

	drop(config);

	info!(
		"Servitor is running and listening on {}",
		listener.local_addr()?
	);

	let app = controllers::create_router(state.clone()).with_state(state);
	axum::serve(listener, app).await.map_err(|e| {
		error!("Server encountered an error: {e}");
		e
	})?;

	Ok(())
}
