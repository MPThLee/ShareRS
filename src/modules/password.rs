use thiserror::Error;

use argon2::password_hash::SaltString;
use argon2::{password_hash, Argon2, PasswordHash, PasswordHasher, PasswordVerifier};

#[allow(dead_code)]
#[derive(Error, Debug)]
pub enum PasswordError {
    #[error("Password encode failed")]
    EncodeError(String),
    #[error("Password verify failed")]
    VerifyError(String),
    #[error("Invalid hash error")]
    InvalidHash(String),
}

#[allow(dead_code)]
pub async fn hash(password: String) -> Result<String, PasswordError> {
    let salt = SaltString::generate(rand::thread_rng());
    Ok(Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| PasswordError::EncodeError(e.to_string()))?
        .to_string())
}

#[allow(dead_code)]
pub async fn verify(password: String, hash: String) -> Result<bool, PasswordError> {
    let hash = PasswordHash::new(&hash).map_err(|e| PasswordError::InvalidHash(e.to_string()))?;

    let res = Argon2::default().verify_password(password.as_bytes(), &hash);

    match res {
        Ok(()) => Ok(true),
        Err(password_hash::Error::Password) => Ok(false),
        Err(e) => Err(PasswordError::VerifyError(e.to_string())),
    }
}
