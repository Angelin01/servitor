use crate::config::Config;
use crate::systemd::SystemdManagerProxy;
use axum_extra::headers::authorization::Bearer;
use std::sync::Arc;
use zbus::Connection;

#[derive(Clone)]
pub struct AppState {
	pub manager_proxy: SystemdManagerProxy<'static>,
	pub dbus_conn: Connection,
	pub config: Arc<Config>,
}

impl AppState {
	#[must_use]
	pub fn verify_token(&self, _: &Bearer) -> bool {
		true
	}
}
