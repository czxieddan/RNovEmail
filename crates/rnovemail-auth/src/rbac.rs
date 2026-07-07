use serde::{Deserialize, Serialize};

use crate::AuthError;

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum Scope {
    AdminUsers,
    AdminDomains,
    AdminProviders,
    AdminMailboxes,
    MailSend,
    MailRead,
    WebhookIngest,
    AuditRead,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct ScopeSet {
    scopes: Vec<Scope>,
}

impl ScopeSet {
    pub fn new(scopes: impl IntoIterator<Item = Scope>) -> Self {
        Self {
            scopes: scopes.into_iter().collect(),
        }
    }

    pub fn require(&self, scope: Scope) -> Result<(), AuthError> {
        match self.scopes.contains(&scope) {
            true => Ok(()),
            false => Err(AuthError::MissingScope),
        }
    }
}
