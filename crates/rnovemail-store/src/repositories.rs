use async_trait::async_trait;
use rnovemail_domain::{
    AuditEvent, DomainName, EmailAddress, InboundMessage, Mailbox, OutboundMessage,
    ProviderAccount, User,
};

use crate::StoreError;

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn put_user(&self, user: User) -> Result<(), StoreError>;
    async fn get_user_by_email(&self, email: &EmailAddress) -> Result<User, StoreError>;
    async fn list_users(&self) -> Result<Vec<User>, StoreError>;
}

#[async_trait]
pub trait DomainRepository: Send + Sync {
    async fn put_domain(&self, domain: DomainName) -> Result<(), StoreError>;
    async fn contains_domain(&self, domain: &DomainName) -> Result<bool, StoreError>;
    async fn list_domains(&self) -> Result<Vec<DomainName>, StoreError>;
}

#[async_trait]
pub trait MailboxRepository: Send + Sync {
    async fn put_mailbox(&self, mailbox: Mailbox) -> Result<(), StoreError>;
    async fn get_mailbox_by_email(&self, email: &EmailAddress) -> Result<Mailbox, StoreError>;
    async fn list_mailboxes(&self) -> Result<Vec<Mailbox>, StoreError>;
}

#[async_trait]
pub trait ProviderRepository: Send + Sync {
    async fn put_provider(&self, provider: ProviderAccount) -> Result<(), StoreError>;
    async fn list_providers(&self) -> Result<Vec<ProviderAccount>, StoreError>;
}

#[async_trait]
pub trait MessageRepository: Send + Sync {
    async fn put_outbound(&self, message: OutboundMessage) -> Result<(), StoreError>;
    async fn put_inbound(&self, message: InboundMessage) -> Result<(), StoreError>;
}

#[async_trait]
pub trait WebhookRepository: Send + Sync {
    async fn remember_event(&self, provider: &str, event_id: &str) -> Result<bool, StoreError>;
}

#[async_trait]
pub trait TokenRepository: Send + Sync {
    async fn put_token_hash(&self, prefix: String, hash: String) -> Result<(), StoreError>;
    async fn get_token_hash(&self, prefix: &str) -> Result<String, StoreError>;
}

#[async_trait]
pub trait AuditRepository: Send + Sync {
    async fn append_audit(&self, event: AuditEvent) -> Result<(), StoreError>;
    async fn list_audit(&self) -> Result<Vec<AuditEvent>, StoreError>;
}

pub trait AppStore:
    UserRepository
    + DomainRepository
    + MailboxRepository
    + ProviderRepository
    + MessageRepository
    + WebhookRepository
    + TokenRepository
    + AuditRepository
{
}

impl<T> AppStore for T where
    T: UserRepository
        + DomainRepository
        + MailboxRepository
        + ProviderRepository
        + MessageRepository
        + WebhookRepository
        + TokenRepository
        + AuditRepository
{
}
