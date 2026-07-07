mod audit;
mod domain;
mod email;
mod mailbox;
mod message;
mod provider;
mod user;

pub use audit::{AuditActor, AuditEvent, AuditResult};
pub use domain::DomainName;
pub use email::EmailAddress;
pub use mailbox::{Mailbox, MailboxId, MailboxStatus};
pub use message::{
    InboundMessage, MessageId, MessageStatus, MessageTimelineEntry, OutboundMessage,
};
pub use provider::{ProviderAccount, ProviderAccountId, ProviderType};
pub use user::{User, UserId, UserRole, UserStatus};

use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum DomainError {
    #[error("value is empty")]
    Empty,
    #[error("value contains unsupported display syntax")]
    DisplaySyntax,
    #[error("value contains invalid whitespace")]
    Whitespace,
    #[error("email address must contain exactly one at sign")]
    EmailShape,
    #[error("domain name is invalid")]
    InvalidDomain,
    #[error("name is too long")]
    TooLong,
}

fn new_uuid() -> Uuid {
    Uuid::now_v7()
}
