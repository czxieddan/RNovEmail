mod errors;
mod provider;
mod resend;
mod routing;

pub use errors::ProviderError;
pub use provider::{
    MailProvider, ProviderEvent, ProviderSendReceipt, ProviderWebhookRequest, SendMailRequest,
    SendMailRequestBuilder, VerifiedWebhook,
};
pub use resend::ResendProvider;
pub use routing::ProviderRegistry;
