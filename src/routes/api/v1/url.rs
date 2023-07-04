use std::{str::FromStr, sync::Arc};

use crate::{
    constraint::*,
    database::models::{url::UrlRequest, Token, TokenId, Url},
    modules::rand::name_gen,
    storage::Storage,
};
use axum::{
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Extension, Json,
};
use serde::Serialize;
use sqlx::PgPool;
use tracing::debug;

use super::{AnyhowError, Error};

#[derive(Serialize)]
struct UploadResponse {
    status: String,
    link: String,
}

pub async fn create(
    Extension(_storage): Extension<Arc<dyn Storage + Send + Sync>>,
    Extension(pool): Extension<PgPool>,
    header: HeaderMap,
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

    let max_views = header
        .get(HEADER_X_OPT_MAX_VIEWS)
        .map(|f| f.to_str().unwrap_or("0").parse::<i64>().unwrap_or(0))
        .and_then(|f| if f == 0 { None } else { Some(f) });

    let destination = match header.get(HEADER_X_DESTINATION) {
        Some(dest) => dest.to_str().unwrap_or("").trim(),
        None => return Ok(Error::bad_request("No destination provided")),
    }
    .to_string();
    if destination.is_empty() {
        return Ok(Error::bad_request("No destination provided"));
    }

    let _name_string = name_gen(10, crate::modules::rand::DBType::Url, None, &pool).await?;
    let _name = _name_string.as_str();
    let name = header
        .get(HEADER_X_OPT_CUSTOM_NAME)
        .map(|f| f.to_str().unwrap_or(_name))
        .unwrap_or(_name)
        .to_string();

    Url::insert(
        UrlRequest {
            name: name.clone(),
            destination,
            max_views,
            user_id: token.user_id,
        },
        &pool,
    )
    .await?;

    Ok((
        StatusCode::OK,
        Json(UploadResponse {
            status: "ok".to_string(),
            link: format!("{}/{}", &dotenvy::var("HOST_BASEURL")?, name),
        }),
    )
        .into_response())
}
