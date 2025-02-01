use crate::models::services::ErrorResponse;
use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use thiserror::Error;
use zbus::names::OwnedErrorName;

#[derive(Debug, Error)]
pub enum ServiceError {
    #[error("Invalid unit name")]
    InvalidUnit,

    #[error("The server encountered an unexpected error")]
    UnexpectedError,
}

impl From<zbus::Error> for ServiceError {
    fn from(e: zbus::Error) -> Self {
        match e {
            zbus::Error::MethodError(name, Some(msg), _) if is_invalid_unit(&name, &msg) => {
                ServiceError::InvalidUnit
            },
            _ => ServiceError::UnexpectedError,
        }
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
            ServiceError::UnexpectedError => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: self.to_string(),
                }),
            ),
        }
        .into_response()
    }
}

fn is_invalid_unit(name: &OwnedErrorName, msg: &str) -> bool {
    name.as_str() == "org.freedesktop.DBus.Error.InvalidArgs"
        && msg.starts_with("Unit name")
        && msg.ends_with("is not valid.")
}
