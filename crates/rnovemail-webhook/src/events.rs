use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct WebhookEventKey {
    provider: String,
    event_id: String,
}

impl WebhookEventKey {
    pub fn new(provider: impl Into<String>, event_id: impl Into<String>) -> Self {
        Self {
            provider: provider.into(),
            event_id: event_id.into(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum WebhookProviderEvent {
    Delivered(WebhookEventKey),
    Bounced(WebhookEventKey),
    Complaint(WebhookEventKey),
    Inbound(WebhookEventKey),
}
