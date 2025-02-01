use crate::models::services::ErrorResponse;
use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum_extra::typed_header::TypedHeaderRejection;
use thiserror::Error;
use zbus::names::OwnedErrorName;

#[derive(Debug, Error)]
pub enum ServiceError {
	#[error("Invalid unit name")]
	InvalidUnit,

	#[error("The server encountered an unexpected error")]
	Unexpected,

	#[error("Invalid authorization token")]
	Unauthorized,
}

impl From<zbus::Error> for ServiceError {
	fn from(e: zbus::Error) -> Self {
		match e {
			zbus::Error::MethodError(name, Some(msg), _) if is_invalid_unit(&name, &msg) => {
				ServiceError::InvalidUnit
			}
			_ => ServiceError::Unexpected,
		}
	}
}

impl From<TypedHeaderRejection> for ServiceError {
	fn from(_: TypedHeaderRejection) -> Self {
		ServiceError::Unauthorized
	}
}

impl IntoResponse for ServiceError {
	fn into_response(self) -> Response {
		match self {
			ServiceError::InvalidUnit => (
				StatusCode::BAD_REQUEST,
				Json(ErrorResponse {
					error: self.to_string(),
				}),
			),
			ServiceError::Unexpected => (
				StatusCode::INTERNAL_SERVER_ERROR,
				Json(ErrorResponse {
					error: self.to_string(),
				}),
			),
			ServiceError::Unauthorized => (
				StatusCode::UNAUTHORIZED,
				Json(ErrorResponse {
					error: self.to_string(),
				}),
			),
		}
		.into_response()
	}
}

fn is_invalid_unit(name: &OwnedErrorName, msg: &str) -> bool {
	let name = name.as_str();
	name == "org.freedesktop.systemd1.NoSuchUnit"
		|| (name == "org.freedesktop.DBus.Error.InvalidArgs"
			&& msg.starts_with("Unit name")
			&& msg.ends_with("is not valid."))
}
