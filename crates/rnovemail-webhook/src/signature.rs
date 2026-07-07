use base64::{Engine, engine::general_purpose::STANDARD};
use hmac::{Hmac, Mac};
use sha2::Sha256;
use subtle::ConstantTimeEq;
use thiserror::Error;

type HmacSha256 = Hmac<Sha256>;
const SVIX_SECRET_PREFIX: &str = "whsec_";
const SVIX_TOLERANCE_SECONDS: u64 = 300;

#[derive(Debug, Error)]
pub enum WebhookSignatureError {
    #[error("webhook signature is missing")]
    Missing,
    #[error("webhook signature is invalid")]
    Invalid,
    #[error("webhook secret is invalid")]
    InvalidSecret,
    #[error("webhook timestamp is outside the allowed window")]
    Expired,
}

#[derive(Clone)]
pub struct SignatureVerifier {
    secret: Vec<u8>,
}

impl SignatureVerifier {
    pub fn new(secret: impl Into<Vec<u8>>) -> Self {
        Self {
            secret: secret.into(),
        }
    }

    pub fn from_svix_secret(secret: impl AsRef<str>) -> Result<Self, WebhookSignatureError> {
        let encoded = secret
            .as_ref()
            .strip_prefix(SVIX_SECRET_PREFIX)
            .ok_or(WebhookSignatureError::InvalidSecret)?;
        let decoded = STANDARD
            .decode(encoded)
            .map_err(|_| WebhookSignatureError::InvalidSecret)?;
        Ok(Self::new(decoded))
    }

    pub fn verify(
        &self,
        id: &str,
        timestamp: &str,
        body: &[u8],
        signature: &str,
    ) -> Result<(), WebhookSignatureError> {
        let now = current_unix_timestamp()?;
        self.verify_at(id, timestamp, body, signature, now)
    }

    pub fn verify_at(
        &self,
        id: &str,
        timestamp: &str,
        body: &[u8],
        signature: &str,
        now: u64,
    ) -> Result<(), WebhookSignatureError> {
        reject_empty_signature(signature)?;
        reject_stale_timestamp(timestamp, now)?;
        let expected = self.sign(id, timestamp, body)?;
        compare_signature(&expected, signature)
    }

    fn sign(
        &self,
        id: &str,
        timestamp: &str,
        body: &[u8],
    ) -> Result<Vec<u8>, WebhookSignatureError> {
        let mut mac =
            HmacSha256::new_from_slice(&self.secret).map_err(|_| WebhookSignatureError::Invalid)?;
        mac.update(id.as_bytes());
        mac.update(b".");
        mac.update(timestamp.as_bytes());
        mac.update(b".");
        mac.update(body);
        Ok(mac.finalize().into_bytes().to_vec())
    }
}

fn reject_empty_signature(signature: &str) -> Result<(), WebhookSignatureError> {
    match signature.is_empty() {
        true => Err(WebhookSignatureError::Missing),
        false => Ok(()),
    }
}

fn reject_stale_timestamp(timestamp: &str, now: u64) -> Result<(), WebhookSignatureError> {
    let signed_at = timestamp
        .parse::<u64>()
        .map_err(|_| WebhookSignatureError::Invalid)?;
    match now.abs_diff(signed_at) <= SVIX_TOLERANCE_SECONDS {
        true => Ok(()),
        false => Err(WebhookSignatureError::Expired),
    }
}

fn compare_signature(expected: &[u8], signature: &str) -> Result<(), WebhookSignatureError> {
    for candidate in signature.split_whitespace().filter_map(parse_v1_signature) {
        if bool::from(candidate.ct_eq(expected)) {
            return Ok(());
        }
    }
    Err(WebhookSignatureError::Invalid)
}

fn parse_v1_signature(value: &str) -> Option<Vec<u8>> {
    let (version, signature) = value.split_once(',')?;
    match version == "v1" {
        true => STANDARD.decode(signature).ok(),
        false => None,
    }
}

fn current_unix_timestamp() -> Result<u64, WebhookSignatureError> {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .map_err(|_| WebhookSignatureError::Invalid)
}
