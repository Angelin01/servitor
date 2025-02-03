use std::collections::HashSet;
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
	pub allowed_services: Option<HashSet<String>>,
	token_hash: Option<Arc<PasswordHash<'static>>>,
}

impl AppState {
	pub fn new(
		manager_proxy: SystemdManagerProxy<'static>,
		dbus_conn: Connection,
		allowed_services: Option<HashSet<String>>,
		token_hash: Option<Arc<PasswordHash<'static>>>,
	) -> Self {
		Self {
			manager_proxy,
			dbus_conn,
			token_hash,
			allowed_services,
		}
	}

	pub fn has_auth(&self) -> bool {
		self.token_hash.is_some()
	}

	pub fn verify_token(&self, bearer: &Bearer) -> Result<()> {
		Ok(match self.token_hash.as_ref() {
			None => (),
			Some(hash) => {
				let algs: &[&dyn PasswordVerifier] = &[&Argon2::default(), &Pbkdf2, &Scrypt];
				hash.verify_password(algs, bearer.token())?
			}
		})
	}
}
