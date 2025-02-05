use crate::config::{Config, DbusScope};
use crate::middleware::auth;
use anyhow::Result;
use log::{error, info};
use state::AppState;
use std::net::SocketAddr;
use std::sync::Arc;
use systemd::SystemdManagerProxy;
use tokio::net::TcpListener;
use tokio::select;
use tokio::signal::unix::{SignalKind, signal};
use tokio::sync::watch;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::filter::LevelFilter;
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
	tracing_subscriber::fmt()
		.with_env_filter(EnvFilter::builder()
			.with_default_directive(LevelFilter::INFO.into())
			.with_env_var("SERV_LOG_LEVEL")
			.from_env_lossy())
		.init();

	let name = env!("CARGO_PKG_NAME");
	let version = env!("CARGO_PKG_VERSION");
	info!("Starting {name} version v{version}...");

	let config = Config::from_envs().map_err(|e| {
		error!("Failed to load configuration: {e}");
		e
	})?;

	let dbus_conn = create_dbus_conn(&config).await?;
	let manager_proxy = create_systemd_manager(dbus_conn.clone()).await?;
	let listener = bind_listener(&config).await?;
	let token_hash = match &config.auth_token {
		None => None,
		Some(t) => {
			let x = auth::parse_token_hash(t).map_err(|e| {
				error!("Failed to read token hash: {e}");
				e
			})?;
			Some(Arc::new(x))
		},
	};

	let state = AppState::new(
		manager_proxy,
		dbus_conn.clone(),
		config.allowlist.clone(),
		token_hash,
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

	let (shutdown_tx, mut shutdown_rx) = watch::channel(());

	let app = controllers::create_router(state.clone()).with_state(state);
	let server = axum::serve(
		listener,
		app.into_make_service_with_connect_info::<SocketAddr>(),
	);

	select! {
		biased;

		_ = server.with_graceful_shutdown(async move {
			let _ = shutdown_rx.changed().await;
		}) => { info!("Axum server shutting down..."); }
		_ = handle_signals(shutdown_tx) => {}
	}

	dbus_conn.graceful_shutdown().await;

	Ok(())
}

async fn bind_listener(config: &Config) -> Result<TcpListener> {
	let listener = TcpListener::bind(config.bind_address.as_str())
		.await
		.map_err(|e| {
			error!("Failed to bind to {}: {e}", config.bind_address);
			e
		})?;

	Ok(listener)
}

async fn create_systemd_manager(dbus_conn: Connection) -> Result<SystemdManagerProxy<'static>> {
	let manager_proxy = SystemdManagerProxy::new(&dbus_conn).await.map_err(|e| {
		error!("Failed to initialize Systemd manager proxy: {e}");
		e
	})?;
	Ok(manager_proxy)
}

async fn create_dbus_conn(config: &Config) -> Result<Connection> {
	let dbus_conn = match config.dbus_scope {
		DbusScope::Session => {
			info!("Using D-Bus session connection.");
			Connection::session().await
		}
		DbusScope::System => {
			info!("Using D-Bus system connection.");
			Connection::system().await
		}
	}
	.map_err(|e| {
		error!("Could not establish a D-Bus connection: {e}");
		e
	})?;
	Ok(dbus_conn)
}

async fn handle_signals(shutdown_tx: watch::Sender<()>) {
	let mut sigterm = signal(SignalKind::terminate()).expect("Failed to install SIGTERM handler");
	let mut sigint = signal(SignalKind::interrupt()).expect("Failed to install SIGINT handler");
	select! {
		_ = sigterm.recv() => (),
		_ = sigint.recv() => (),
	}
	info!("Received shutdown signal, shutting down...");
	let _ = shutdown_tx.send(());
}
