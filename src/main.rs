#![feature(trivial_bounds)]

use crate::config::Config;
use anyhow::Result;
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
	// TODO: prettier error messages with error!
	let dbus_conn = Connection::session().await?;
	let manager_proxy = SystemdManagerProxy::new(&dbus_conn).await?;
	let config = Arc::new(Config::from_envs()?);
	let listener = TcpListener::bind(config.bind_address.as_str()).await?;

	let state = AppState {
		manager_proxy,
		dbus_conn,
		config,
	};

	let app = controllers::create_router(state.clone()).with_state(state);
	println!("Servitor running");
	axum::serve(listener, app).await?;

	Ok(())
}
