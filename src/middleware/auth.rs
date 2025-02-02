use std::net::SocketAddr;
use crate::errors::ServiceError;
use crate::state::AppState;
use anyhow::Result;
use axum::{
	extract::{Request, State},
	middleware::Next,
	response::Response,
};
use axum::extract::ConnectInfo;
use axum_extra::TypedHeader;
use axum_extra::extract::WithRejection;
use axum_extra::headers::Authorization;
use axum_extra::headers::authorization::Bearer;
use log::info;
use password_hash::PasswordHash;

pub fn parse_token_hash(auth_token: &str) -> Result<PasswordHash<'static>> {
	let static_token = Box::leak(auth_token.to_owned().into_boxed_str());
	PasswordHash::new(static_token)
		.map_err(|e| anyhow::Error::from(e))
}

pub async fn auth_middleware(
	State(state): State<AppState>,
	ConnectInfo(addr): ConnectInfo<SocketAddr>,
	WithRejection(TypedHeader(Authorization(bearer)), _): WithRejection<
		TypedHeader<Authorization<Bearer>>,
		ServiceError,
	>,
	req: Request,
	next: Next,
) -> Result<Response, ServiceError> {
	let verification_result = tokio::task::spawn_blocking(move || {
		state.verify_token(&bearer)
	})
		.await
		.map_err(|_| ServiceError::Unexpected)?;

	match verification_result {
		Ok(_) => Ok(next.run(req).await),
		Err(e) => {
			let ip = addr.ip().to_string();
			info!("Denied authentication to IP {ip}: {e}");
			Err(ServiceError::Unauthorized)
		}
	}
}
