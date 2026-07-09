use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{EmailAddress, MailboxId, ProviderAccountId, new_uuid};

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct MessageId(Uuid);

impl MessageId {
    pub fn new() -> Self {
        Self(new_uuid())
    }
}

impl Default for MessageId {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum MessageStatus {
    Accepted,
    Sent,
    Delivered,
    Bounced,
    Complained,
    Failed,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum MessageDirection {
    Inbound,
    Outbound,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct MessageTimelineEntry {
    pub status: MessageStatus,
    pub at: DateTime<Utc>,
    pub note: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct OutboundMessage {
    pub id: MessageId,
    pub provider_account_id: ProviderAccountId,
    #[serde(default)]
    pub provider_message_id: Option<String>,
    pub from: EmailAddress,
    pub to: Vec<EmailAddress>,
    pub subject: String,
    pub text: String,
    pub status: MessageStatus,
    pub timeline: Vec<MessageTimelineEntry>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct InboundMessage {
    pub id: MessageId,
    pub mailbox_id: MailboxId,
    #[serde(default)]
    pub provider_account_id: Option<ProviderAccountId>,
    pub provider_event_id: String,
    pub from: EmailAddress,
    pub subject: String,
    pub text: String,
    pub received_at: DateTime<Utc>,
    #[serde(default)]
    pub detail: Option<InboundMessageDetail>,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct InboundMessageDetail {
    pub from: String,
    pub to: Vec<String>,
    pub cc: Vec<String>,
    pub bcc: Vec<String>,
    pub reply_to: Vec<String>,
    pub subject: String,
    pub text: String,
    pub html: Option<String>,
    pub headers: Vec<InboundMessageHeader>,
    pub attachments: Vec<InboundMessageAttachment>,
    pub raw: Option<InboundMessageRaw>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct InboundMessageHeader {
    pub name: String,
    pub value: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct InboundMessageAttachment {
    pub filename: String,
    pub content_type: String,
    pub content_disposition: String,
    pub content_id: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct InboundMessageRaw {
    pub download_url: String,
    pub expires_at: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct MessageUserState {
    pub user_email: EmailAddress,
    pub direction: MessageDirection,
    pub provider_message_id: String,
    pub starred: bool,
    pub deleted: bool,
}

impl MessageUserState {
    pub fn new(
        user_email: EmailAddress,
        direction: MessageDirection,
        provider_message_id: impl Into<String>,
    ) -> Self {
        Self {
            user_email,
            direction,
            provider_message_id: provider_message_id.into(),
            starred: false,
            deleted: false,
        }
    }

    pub fn set_starred(&mut self, starred: bool) {
        self.starred = starred;
    }

    pub fn mark_deleted(&mut self) {
        self.deleted = true;
    }
}
