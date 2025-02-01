use crate::systemd::SystemdManagerProxy;
use zbus::Connection;

#[derive(Clone)]
pub struct AppState {
	pub manager_proxy: SystemdManagerProxy<'static>,
	pub dbus_conn: Connection,
}
