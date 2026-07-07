use base64::{Engine, engine::general_purpose::STANDARD};
use hmac::{Hmac, Mac};
use sha2::Sha256;
use thiserror::Error;

type HmacSha256 = Hmac<Sha256>;

#[derive(Debug, Error)]
pub enum WebhookSignatureError {
    #[error("webhook signature is missing")]
    Missing,
    #[error("webhook signature is invalid")]
    Invalid,
}

pub struct SignatureVerifier {
    secret: Vec<u8>,
}

impl SignatureVerifier {
    pub fn new(secret: impl Into<Vec<u8>>) -> Self {
        Self {
            secret: secret.into(),
        }
    }

    pub fn verify(
        &self,
        timestamp: &str,
        body: &[u8],
        signature: &str,
    ) -> Result<(), WebhookSignatureError> {
        reject_empty_signature(signature)?;
        let expected = self.sign(timestamp, body)?;
        compare_signature(&expected, signature)
    }

    fn sign(&self, timestamp: &str, body: &[u8]) -> Result<String, WebhookSignatureError> {
        let mut mac =
            HmacSha256::new_from_slice(&self.secret).map_err(|_| WebhookSignatureError::Invalid)?;
        mac.update(timestamp.as_bytes());
        mac.update(b".");
        mac.update(body);
        Ok(STANDARD.encode(mac.finalize().into_bytes()))
    }
}

fn reject_empty_signature(signature: &str) -> Result<(), WebhookSignatureError> {
    match signature.is_empty() {
        true => Err(WebhookSignatureError::Missing),
        false => Ok(()),
    }
}

fn compare_signature(expected: &str, signature: &str) -> Result<(), WebhookSignatureError> {
    match expected == signature || hex::encode(expected) == signature {
        true => Ok(()),
        false => Err(WebhookSignatureError::Invalid),
    }
}
