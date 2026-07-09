use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rnovemail_domain::{
    EmailAddress, InboundMessageDetail, MessageId, MessageStatus, ProviderType,
};
use serde::{Deserialize, Serialize};

use crate::ProviderError;

#[async_trait]
pub trait MailProvider: Send + Sync {
    fn provider_type(&self) -> ProviderType;
    async fn send(&self, request: SendMailRequest) -> Result<ProviderSendReceipt, ProviderError>;
    fn verify_webhook(
        &self,
        request: ProviderWebhookRequest,
    ) -> Result<VerifiedWebhook, ProviderError>;
    fn map_webhook(&self, verified: VerifiedWebhook) -> Result<Vec<ProviderEvent>, ProviderError>;
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct SendMailRequest {
    from: EmailAddress,
    to: Vec<EmailAddress>,
    subject: String,
    text: String,
    html: Option<String>,
}

impl SendMailRequest {
    pub fn builder() -> SendMailRequestBuilder {
        SendMailRequestBuilder::default()
    }

    pub fn from(&self) -> &EmailAddress {
        &self.from
    }

    pub fn to(&self) -> &[EmailAddress] {
        &self.to
    }

    pub fn subject(&self) -> &str {
        &self.subject
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn html(&self) -> Option<&str> {
        self.html.as_deref()
    }
}

#[derive(Default)]
pub struct SendMailRequestBuilder {
    from: Option<EmailAddress>,
    to: Vec<EmailAddress>,
    subject: Option<String>,
    text: Option<String>,
    html: Option<String>,
}

impl SendMailRequestBuilder {
    pub fn from(mut self, from: EmailAddress) -> Self {
        self.from = Some(from);
        self
    }

    pub fn to(mut self, to: impl IntoIterator<Item = EmailAddress>) -> Self {
        self.to = to.into_iter().collect();
        self
    }

    pub fn subject(mut self, subject: impl Into<String>) -> Self {
        self.subject = Some(subject.into());
        self
    }

    pub fn text(mut self, text: impl Into<String>) -> Self {
        self.text = Some(text.into());
        self
    }

    pub fn html(mut self, html: impl Into<String>) -> Self {
        self.html = Some(html.into());
        self
    }

    pub fn build(self) -> Result<SendMailRequest, ProviderError> {
        let from = self.from.ok_or(ProviderError::MissingField("from"))?;
        let subject = self.subject.ok_or(ProviderError::MissingField("subject"))?;
        let text = self.text.ok_or(ProviderError::MissingField("text"))?;
        reject_empty_recipients(&self.to)?;
        Ok(SendMailRequest {
            from,
            to: self.to,
            subject,
            text,
            html: self.html,
        })
    }
}

fn reject_empty_recipients(recipients: &[EmailAddress]) -> Result<(), ProviderError> {
    match recipients.is_empty() {
        true => Err(ProviderError::EmptyRecipients),
        false => Ok(()),
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ProviderSendReceipt {
    pub message_id: MessageId,
    pub provider_message_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProviderOutboundHistoryItem {
    pub provider_message_id: String,
    pub from: EmailAddress,
    pub to: Vec<EmailAddress>,
    pub subject: String,
    pub text: String,
    pub html: Option<String>,
    pub status: MessageStatus,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProviderInboundHistoryItem {
    pub provider_message_id: String,
    pub from: EmailAddress,
    pub to: Vec<EmailAddress>,
    pub subject: String,
    pub text: String,
    pub detail: InboundMessageDetail,
    pub received_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProviderWebhookRequest {
    pub id: String,
    pub signature: String,
    pub timestamp: String,
    pub body: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VerifiedWebhook {
    pub provider: ProviderType,
    pub body: Vec<u8>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum ProviderEvent {
    Delivered {
        provider_event_id: String,
    },
    Bounced {
        provider_event_id: String,
    },
    Complained {
        provider_event_id: String,
    },
    Inbound {
        provider_event_id: String,
        from: EmailAddress,
        to: Vec<EmailAddress>,
        subject: String,
        text: String,
    },
}
