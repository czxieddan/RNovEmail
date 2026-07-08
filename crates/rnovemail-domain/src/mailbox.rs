use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{EmailAddress, UserId, new_uuid};

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct MailboxId(Uuid);

impl MailboxId {
    pub fn new() -> Self {
        Self(new_uuid())
    }
}

impl Default for MailboxId {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum MailboxStatus {
    Active,
    Disabled,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Mailbox {
    id: MailboxId,
    owner_id: UserId,
    address: EmailAddress,
    status: MailboxStatus,
    inbound_enabled: bool,
    outbound_enabled: bool,
}

impl Mailbox {
    pub fn assign(owner_id: UserId, address: EmailAddress) -> Self {
        Self {
            id: MailboxId::new(),
            owner_id,
            address,
            status: MailboxStatus::Active,
            inbound_enabled: true,
            outbound_enabled: true,
        }
    }

    pub fn id(&self) -> MailboxId {
        self.id
    }

    pub fn owner_id(&self) -> UserId {
        self.owner_id
    }

    pub fn address(&self) -> &EmailAddress {
        &self.address
    }

    pub fn status(&self) -> MailboxStatus {
        self.status
    }

    pub fn inbound_enabled(&self) -> bool {
        self.inbound_enabled
    }

    pub fn outbound_enabled(&self) -> bool {
        self.outbound_enabled
    }

    pub fn set_status(&mut self, status: MailboxStatus) {
        self.status = status;
    }

    pub fn set_inbound_enabled(&mut self, enabled: bool) {
        self.inbound_enabled = enabled;
    }

    pub fn set_outbound_enabled(&mut self, enabled: bool) {
        self.outbound_enabled = enabled;
    }
}
