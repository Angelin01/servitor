use crate::config::Config;
use anyhow::Result;
use state::AppState;
use std::sync::Arc;
use systemd::SystemdManagerProxy;
use tokio::net::TcpListener;
use zbus::Connection;
use crate::middleware::auth;

mod config;
mod controllers;
mod errors;
mod middleware;
mod models;
mod state;
mod systemd;

#[tokio::main]
async fn main() -> Result<()> {
	// TODO: prettier error messages with error!
	let config = Config::from_envs()?;
	let dbus_conn = Connection::session().await?;
	let manager_proxy = SystemdManagerProxy::new(&dbus_conn).await?;
	let listener = TcpListener::bind(config.bind_address.as_str()).await?;

	let password_hash = auth::read_password_hash(config.auth_token.as_deref())?
		.map(Arc::new);

	let state = AppState::new(
		manager_proxy,
		dbus_conn,
		password_hash,
		config.allowlist.clone(),
	);

	drop(config.allowlist);

	let app = controllers::create_router(state.clone()).with_state(state);
	println!("Servitor running");
	axum::serve(listener, app).await?;

	Ok(())
}
