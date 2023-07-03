use std::sync::Arc;

use crate::{
    database::models::{File, Url},
    modules::template::default_context,
    storage::Storage,
};
use axum::{
    extract::Path,
    http::{header, StatusCode},
    response::{Html, IntoResponse, Redirect, Response},
    Extension,
};
use sqlx::PgPool;
use std::path::Path as StdPath;
use tera::Tera;
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
                        error!(
                            "File not found while DB has an file hash '{}': {} / {:?}",
                            hash, err, file
                        );
                        return (StatusCode::NOT_FOUND, "Not found".to_string()).into_response();
                    }
                };

                tokio::spawn(async move { File::increase_views(file.id, &pool).await });
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
                    bytes,
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

pub async fn index(
    Extension(tera): Extension<Tera>,
    Extension(pool): Extension<PgPool>,
) -> Response {
    let mut context = default_context();
    context.insert("total_files", &File::get_count(&pool).await.unwrap_or(0));
    context.insert("total_urls", &Url::get_count(&pool).await.unwrap_or(0));

    match tera.render("index.html", &context) {
        Ok(html) => Html(html).into_response(),
        Err(err) => {
            error!("Error while render index template: {}", err);
            (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error").into_response()
        }
    }
}
