use anyhow::{Result, anyhow};
use std::collections::HashSet;
use std::env::var;
use std::str::FromStr;

pub enum DbusScope {
	Session,
	System,
}

impl FromStr for DbusScope {
	type Err = anyhow::Error;
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s.to_ascii_lowercase().as_str() {
			"session" => Ok(DbusScope::Session),
			"system" => Ok(DbusScope::System),
			s => Err(anyhow!("Unknown dbus scope {s}")),
		}
	}
}

pub struct Config {
	pub bind_address: String,
	pub auth_token: Option<String>,
	pub allowlist: Option<HashSet<String>>,
	pub dbus_scope: DbusScope,
}

impl Config {
	pub fn from_envs() -> Result<Self> {
		let bind_address = var("SERV_BIND_ADDRESS").unwrap_or("127.0.0.1:8008".into());
		let auth_enabled = var("SERV_AUTH_ENABLED").map_or(true, |v| v.parse().unwrap_or(true));
		let auth_token = if auth_enabled {
			Some(var("SERV_AUTH_TOKEN").map_err(|_| {
				anyhow!("Auth is enabled and failed to read SERV_AUTH_TOKEN env var")
			})?)
		} else {
			None
		};
		let allowlist = var("SERV_ALLOWLIST").ok().map(|v| {
			v.split(',')
				.map(str::trim)
				.filter(|s| !s.is_empty())
				.map(String::from)
				.collect()
		});

		let dbus_scope = var("SERV_DBUS_SCOPE")
			.ok()
			.map(|s| s.parse())
			.transpose()?
			.unwrap_or(DbusScope::Session);

		Ok(Self {
			bind_address,
			auth_token,
			allowlist,
			dbus_scope,
		})
	}
}
