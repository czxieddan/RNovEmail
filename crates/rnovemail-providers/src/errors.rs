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
    ProviderRejected,
    #[error("provider payload is invalid")]
    InvalidPayload,
}
