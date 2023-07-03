use serde::{Deserialize, Serialize};
use sqlx::sqlx_macros::Type;
use uuid::Uuid;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Type, Serialize, Deserialize, Hash)]
#[sqlx(transparent)]
pub struct UserId(pub Uuid);

#[derive(Copy, Clone, Debug, PartialEq, Eq, Type, Serialize, Deserialize, Hash)]
#[sqlx(transparent)]
pub struct TokenId(pub Uuid);

#[derive(Copy, Clone, Debug, PartialEq, Eq, Type, Serialize, Deserialize, Hash)]
#[sqlx(transparent)]
pub struct FileId(pub Uuid);

#[derive(Copy, Clone, Debug, PartialEq, Eq, Type, Serialize, Deserialize, Hash)]
#[sqlx(transparent)]
pub struct UrlId(pub Uuid);
