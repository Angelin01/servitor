use crate::errors::ServiceError;
use crate::state::AppState;
use anyhow::Result;
use axum::{
	extract::{Request, State},
	middleware::Next,
	response::Response,
};
use axum_extra::TypedHeader;
use axum_extra::extract::WithRejection;
use axum_extra::headers::Authorization;
use axum_extra::headers::authorization::Bearer;
use password_hash::PasswordHash;

pub fn read_password_hash(auth_token: Option<&str>) -> Result<Option<PasswordHash<'static>>> {
	match auth_token {
		None => Ok(None),
		Some(t) => {
			let static_token = Box::leak(t.to_owned().into_boxed_str());
			PasswordHash::new(static_token)
				.map(Some)
				.map_err(|e| anyhow::anyhow!("Failed to parse password hash: {}", e))
		}
	}
}


pub async fn auth_middleware(
	State(state): State<AppState>,
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
		Err(_) => Err(ServiceError::Unauthorized),
	}
}
