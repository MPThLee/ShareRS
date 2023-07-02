mod api;

use std::sync::Arc;

use crate::storage::Storage;
use axum::{
    extract::Path,
    http::status::StatusCode,
    response::{IntoResponse, Response},
    Extension,
};
use std::path::Path as StdPath;

pub async fn get_hash(
    Path(hash): Path<String>,
    Extension(storage): Extension<Arc<dyn Storage + Send + Sync>>,
) -> Response {
    let path = StdPath::new(&hash);

    match path.extension() {
        Some(_ext) => {
            if !(storage.exists(&hash).await) {
                return (StatusCode::NOT_FOUND, "Not found").into_response();
            }

            (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error").into_response()
        }
        None => (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error").into_response(),
    }
}
