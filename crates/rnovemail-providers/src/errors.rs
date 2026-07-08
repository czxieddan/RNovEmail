use thiserror::Error;

#[derive(Debug, Error)]
pub enum ProviderError {
    #[error("no active provider account for sender domain")]
    NoProviderForDomain,
    #[error("send request is missing {0}")]
    MissingField(&'static str),
    #[error("recipient list is empty")]
    EmptyRecipients,
    #[error("provider webhook signature is invalid")]
    InvalidSignature,
    #[error("provider response was rejected")]
    ProviderRejected { status: Option<u16> },
    #[error("provider payload is invalid")]
    InvalidPayload,
}

impl ProviderError {
    pub fn provider_rejected() -> Self {
        Self::ProviderRejected { status: None }
    }

    pub fn provider_rejected_status(status: u16) -> Self {
        Self::ProviderRejected {
            status: Some(status),
        }
    }
}
