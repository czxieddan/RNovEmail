use thiserror::Error;

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("api token is invalid")]
    InvalidToken,
    #[error("scope is missing")]
    MissingScope,
    #[error("secret encryption failed")]
    SecretEncryption,
    #[error("secret decryption failed")]
    SecretDecryption,
}
