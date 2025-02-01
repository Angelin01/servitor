use axum_extra::headers::authorization::Bearer;
use crate::systemd::SystemdManagerProxy;
use zbus::Connection;

#[derive(Clone)]
pub struct AppState {
	pub manager_proxy: SystemdManagerProxy<'static>,
	pub dbus_conn: Connection,
}

impl AppState {
	#[must_use]
	pub fn verify_token(&self, _: &Bearer) -> bool {
		true
	}
}
