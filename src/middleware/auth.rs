use crate::errors::ServiceError;
use crate::state::AppState;
use axum::{
	extract::{Request, State},
	middleware::Next,
	response::Response,
};
use axum_extra::TypedHeader;
use axum_extra::extract::WithRejection;
use axum_extra::headers::Authorization;
use axum_extra::headers::authorization::Bearer;

pub async fn auth_middleware(
	State(state): State<AppState>,
	WithRejection(TypedHeader(Authorization(bearer)), _): WithRejection<
		TypedHeader<Authorization<Bearer>>,
		ServiceError,
	>,
	req: Request,
	next: Next,
) -> Result<Response, ServiceError> {
	println!("Bearer: {bearer:?}");
	println!("Token: {}", bearer.token());

	if state.verify_token(&bearer) {
		Ok(next.run(req).await)
	} else {
		Err(ServiceError::Unauthorized)
	}
}
