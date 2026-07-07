use argon2::{
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
    password_hash::{SaltString, rand_core::OsRng},
};
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use rand::RngCore;
use secrecy::{ExposeSecret, SecretString};
use serde::{Deserialize, Serialize};

use crate::AuthError;

#[derive(Clone)]
pub struct ApiToken {
    prefix: String,
    secret: SecretString,
}

impl ApiToken {
    pub fn prefix(&self) -> &str {
        &self.prefix
    }

    pub fn expose_once(&self) -> String {
        format!("{}.{}", self.prefix, self.secret.expose_secret())
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ApiTokenHash {
    prefix: String,
    hash: String,
}

impl ApiTokenHash {
    pub fn prefix(&self) -> &str {
        &self.prefix
    }

    pub fn verify(&self, presented: &str) -> Result<(), AuthError> {
        let secret = split_presented_secret(&self.prefix, presented)?;
        verify_hash(&self.hash, secret)
    }
}

pub struct TokenGenerator;

impl TokenGenerator {
    pub fn generate() -> Result<(ApiToken, ApiTokenHash), AuthError> {
        let prefix = random_fragment(8);
        let secret = SecretString::new(random_fragment(32));
        let hash = hash_secret(secret.expose_secret())?;
        let token = ApiToken {
            prefix: prefix.clone(),
            secret,
        };
        Ok((token, ApiTokenHash { prefix, hash }))
    }
}

fn split_presented_secret<'a>(prefix: &str, presented: &'a str) -> Result<&'a str, AuthError> {
    let Some((actual_prefix, secret)) = presented.split_once('.') else {
        return Err(AuthError::InvalidToken);
    };
    match actual_prefix == prefix {
        true => Ok(secret),
        false => Err(AuthError::InvalidToken),
    }
}

fn random_fragment(bytes: usize) -> String {
    let mut data = vec![0_u8; bytes];
    rand::rngs::OsRng.fill_bytes(&mut data);
    URL_SAFE_NO_PAD.encode(data)
}

fn hash_secret(secret: &str) -> Result<String, AuthError> {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(secret.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|_| AuthError::InvalidToken)
}

fn verify_hash(hash: &str, secret: &str) -> Result<(), AuthError> {
    let parsed = PasswordHash::new(hash).map_err(|_| AuthError::InvalidToken)?;
    Argon2::default()
        .verify_password(secret.as_bytes(), &parsed)
        .map_err(|_| AuthError::InvalidToken)
}
