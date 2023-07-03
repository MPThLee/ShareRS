mod api;

use std::sync::Arc;

use crate::{
    database::models::{File, Url},
    storage::Storage,
};
use axum::{
    extract::Path,
    http::{header, StatusCode},
    response::{IntoResponse, Redirect, Response},
    Extension,
};
use sqlx::PgPool;
use std::path::Path as StdPath;
use tracing::{debug, error};

pub async fn get_hash(
    Path(hash): Path<String>,
    Extension(storage): Extension<Arc<dyn Storage + Send + Sync>>,
    Extension(pool): Extension<PgPool>,
) -> Response {
    let path = StdPath::new(&hash);

    // Let's treat non-dot hases as url redirect, others are files.
    match path.extension() {
        Some(_ext) => {
            if let Ok(Some(file)) = File::get_by_name(&hash, &pool).await {
                let bytes = match storage.get(&file.id.0.to_string()).await {
                    Ok(file) => file,
                    Err(err) => {
                        error!("File not found while DB has an file hash '{}': {} / {:?}", hash, err, file);
                        return (StatusCode::NOT_FOUND, format!("Not found"))
                            .into_response()
                    }
                };

                (
                    [
                        (
                            header::CONTENT_TYPE,
                            file.mime.unwrap_or("application/octet-stream".to_string()),
                        ),
                        (
                            header::CONTENT_DISPOSITION,
                            format!(
                                "attachment; filename=\"{}\"",
                                file.original_name.unwrap_or(file.name)
                            ),
                        ),
                    ],
                    bytes.to_vec(),
                )
                    .into_response()
            } else {
                debug!("Could not found valid file for hash '{}'.", hash);
                (StatusCode::NOT_FOUND, "Not found").into_response()
            }
        }
        // Url redirect
        None => {
            if let Ok(Some(url)) = Url::get_by_name(&hash, &pool).await {
                debug!(
                    "Redirect to {}, UrlId: {}, Hash: {}",
                    url.destination, url.id.0, hash
                );
                // Fire and forgot the increse views
                tokio::spawn(async move { Url::increase_views(url.id, &pool).await });
                Redirect::temporary(&url.destination).into_response()
            } else {
                debug!("Could not found valid redirect for hash '{}'.", hash);
                (StatusCode::NOT_FOUND, "Not found").into_response()
            }
        }
    }
}
