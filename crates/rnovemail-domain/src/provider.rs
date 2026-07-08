use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{DomainName, new_uuid};

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct ProviderAccountId(Uuid);

impl ProviderAccountId {
    pub fn new() -> Self {
        Self(new_uuid())
    }
}

impl Default for ProviderAccountId {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum ProviderType {
    Resend,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ProviderAccount {
    id: ProviderAccountId,
    provider_type: ProviderType,
    name: String,
    domains: Vec<DomainName>,
    enabled: bool,
}

impl ProviderAccount {
    pub fn new(
        provider_type: ProviderType,
        name: impl Into<String>,
        domains: impl IntoIterator<Item = DomainName>,
    ) -> Self {
        Self {
            id: ProviderAccountId::new(),
            provider_type,
            name: name.into(),
            domains: domains.into_iter().collect(),
            enabled: true,
        }
    }

    pub fn id(&self) -> ProviderAccountId {
        self.id
    }

    pub fn provider_type(&self) -> ProviderType {
        self.provider_type
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn serves_domain(&self, domain: &DomainName) -> bool {
        self.enabled && self.domains.iter().any(|candidate| candidate == domain)
    }

    pub fn domains(&self) -> &[DomainName] {
        &self.domains
    }

    pub fn enabled(&self) -> bool {
        self.enabled
    }

    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }

    pub fn set_domains(&mut self, domains: Vec<DomainName>) {
        self.domains = domains;
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
}
