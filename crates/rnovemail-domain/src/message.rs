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
    pub provider_event_id: String,
    pub from: EmailAddress,
    pub subject: String,
    pub text: String,
    pub received_at: DateTime<Utc>,
}
