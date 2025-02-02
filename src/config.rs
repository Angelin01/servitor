use anyhow::Result;
use std::env::var;

pub struct Config {
	pub bind_address: String,
	pub auth_token: Option<String>, // TODO: Make it a password verifier instead
	pub allowlist: Option<Vec<String>>,
}

impl Config {
	pub fn from_envs() -> Result<Self> {
		let bind_address = var("SERV_BIND_ADDRESS").unwrap_or("0.0.0.0".into());
		let auth_enabled = var("SERV_AUTH_ENABLED").map_or(true, |v| v.parse().unwrap_or(true));
		let auth_token = if auth_enabled {
			Some(var("SERV_AUTH_TOKEN")?)
		}
		else {
			None
		};
		let allowlist = var("SERV_ALLOWLIST")
			.ok()
			.map(|v| v.split(',').map(str::trim).map(String::from).collect());

		Ok(Self {
			bind_address,
			auth_token,
			allowlist,
		})
	}
}
