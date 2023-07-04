use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use tracing::error;

pub mod file;
pub mod url;

#[derive(Serialize)]
struct Error {
    status: String,
    message: String,
}

impl Error {
    fn unauthorized<S: Into<String>>(message: S) -> Response {
        (StatusCode::UNAUTHORIZED, Self::unauthorized_json(message)).into_response()
    }

    fn unauthorized_json<S: Into<String>>(message: S) -> Json<Error> {
        Json(Error {
            status: "Unauthorized".to_string(),
            message: message.into(),
        })
    }

    fn bad_request<S: Into<String>>(message: S) -> Response {
        (StatusCode::BAD_REQUEST, Self::bad_request_json(message)).into_response()
    }

    fn bad_request_json<S: Into<String>>(message: S) -> Json<Error> {
        Json(Error {
            status: "Bad Request".to_string(),
            message: message.into(),
        })
    }

    fn internal_server_error<S: Into<String>>(message: S) -> Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Self::internal_server_error_json(message),
        )
            .into_response()
    }

    fn internal_server_error_json<S: Into<String>>(message: S) -> Json<Error> {
        Json(Error {
            status: "Internal Server Error".to_string(),
            message: message.into(),
        })
    }
}

pub struct AnyhowError(anyhow::Error);

impl IntoResponse for AnyhowError {
    fn into_response(self) -> Response {
        error!("Error on response: {}", &self.0);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Error::internal_server_error(format!("Something went wrong: {:?}", &self.0)),
        )
            .into_response()
    }
}

impl<E> From<E> for AnyhowError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}
