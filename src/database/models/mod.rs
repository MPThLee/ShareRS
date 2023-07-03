use thiserror::Error;

pub mod file;
pub mod ids;
pub mod token;
pub mod url;
pub mod user;

pub use file::File;
pub use ids::*;
pub use token::Token;
pub use url::Url;
pub use user::User;

use crate::modules::password::PasswordError;

#[derive(Error, Debug)]
pub enum DatabaseError {
    #[error("Error while interacting with the database: {0}")]
    Database(#[from] sqlx::error::Error),
    // #[error("Error while trying to generate random ID")]
    // RandomId,
    #[error("Password algorithm failed: {0}")]
    PasswordHash(#[from] PasswordError),
    #[error("A database request failed")]
    Other(String),
    #[error("Error while parsing JSON: {0}")]
    Json(#[from] serde_json::Error),
}
