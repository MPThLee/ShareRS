use std::{str::FromStr, sync::Arc};

use crate::{
    constraint::*,
    database::models::{file::FileDbRequest, File, Token, TokenId},
    modules::{mime::get_mime, rand::name_gen},
    storage::Storage,
};
use anyhow::anyhow;
use axum::{
    extract::Multipart,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Extension, Json,
};
use bytes::Bytes;
use serde::Serialize;
use sqlx::PgPool;
use tracing::debug;

use super::{AnyhowError, Error};

#[derive(Serialize)]
struct UploadResponse {
    status: String,
    link: String,
}

pub async fn upload(
    Extension(storage): Extension<Arc<dyn Storage + Send + Sync>>,
    Extension(pool): Extension<PgPool>,
    header: HeaderMap,
    mut multipart: Multipart,
) -> Result<Response, AnyhowError> {
    let token = match header.get(HEADER_X_TOKEN) {
        Some(token) => match uuid::Uuid::from_str(token.to_str().unwrap_or("")) {
            Ok(uuid) => TokenId(uuid),
            Err(err) => {
                debug!("Invalid uuid token error: {}", err);
                return Ok(Error::bad_request("Invalid token provided"));
            }
        },
        None => return Ok(Error::unauthorized("No token provided")),
    };
    let token = match Token::get_valid_by_id(token, &pool).await? {
        Some(token) => token,
        None => return Ok(Error::unauthorized("No valid token provided")),
    };

    let original_name = header
        .get(HEADER_X_OPT_ORIGINAL_NAME)
        .map(|f| f.to_str().unwrap_or("").to_string());
    let max_views = header
        .get(HEADER_X_OPT_MAX_VIEWS)
        .map(|f| f.to_str().unwrap_or("0").parse::<i64>().unwrap_or(0))
        .and_then(|f| if f == 0 { None } else { Some(f) });

    let mut _file_bytes: Option<Bytes> = None;
    while let Some(field) = multipart.next_field().await? {
        let name = &field.name().unwrap_or("").to_string();
        if name == "file" {
            _file_bytes = Some(field.bytes().await?);
        }
    }

    if _file_bytes.is_none() {
        return Ok(Error::bad_request(
            "Could not found 'file' on multipart body",
        ));
    }

    let file_bytes = _file_bytes.ok_or(anyhow!("Invalid file bytes"))?;
    let mime_type = get_mime(&file_bytes.slice(0..12));
    let mime = mime_type.mime_type().to_string();

    let _name_string = name_gen(
        10,
        crate::modules::rand::DBType::File,
        Some(mime_type.extension().to_string()),
        &pool,
    )
    .await?;
    let _name = _name_string.as_str();
    let name = header
        .get(HEADER_X_OPT_CUSTOM_NAME)
        .map(|f| f.to_str().unwrap_or(_name))
        .unwrap_or(_name)
        .to_string();

    let id = File::insert(
        FileDbRequest {
            name: name.clone(),
            original_name,
            mime: Some(mime),
            max_views,
            user_id: token.user_id,
        },
        &pool,
    )
    .await?;

    storage.put(&id.0.to_string(), file_bytes).await?;
    File::update_processing(id, false, &pool).await?;
    Ok((
        StatusCode::OK,
        Json(UploadResponse {
            status: "ok".to_string(),
            link: format!("{}/{}", &dotenvy::var("HOST_BASEURL")?, name),
        }),
    )
        .into_response())
}
