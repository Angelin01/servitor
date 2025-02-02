use crate::systemd::SystemdManagerProxy;
use anyhow::Result;
use axum_extra::headers::authorization::Bearer;
use password_hash::{PasswordHash, PasswordVerifier};
use std::sync::Arc;
use zbus::Connection;

use argon2::Argon2;
use pbkdf2::Pbkdf2;
use scrypt::Scrypt;

#[derive(Clone)]
pub struct AppState {
	pub manager_proxy: SystemdManagerProxy<'static>,
	pub dbus_conn: Connection,
	password_hash: Option<Arc<PasswordHash<'static>>>,
}

impl AppState {
	pub fn new(
		manager_proxy: SystemdManagerProxy<'static>,
		dbus_conn: Connection,
		password_hash: Option<Arc<PasswordHash<'static>>>,
	) -> Self {
		Self {
			manager_proxy,
			dbus_conn,
			password_hash,
		}
	}

	pub fn has_auth(&self) -> bool {
		self.password_hash.is_some()
	}

	pub fn verify_token(&self, bearer: &Bearer) -> Result<()> {
		Ok(match self.password_hash.as_ref() {
			None => (),
			Some(hash) => {
				let algs: &[&dyn PasswordVerifier] = &[&Argon2::default(), &Pbkdf2, &Scrypt];
				hash.verify_password(algs, bearer.token())?
			}
		})
	}
}
