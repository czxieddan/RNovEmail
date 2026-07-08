use serde::{Deserialize, Serialize};
use uuid::Uuid;
use zeroize::Zeroize;

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

#[derive(Clone, Deserialize, Eq, PartialEq, Serialize)]
pub struct ProviderSecret(String);

impl ProviderSecret {
    fn new(value: impl Into<String>) -> Option<Self> {
        let value = value.into().trim().to_string();
        match value.is_empty() {
            true => None,
            false => Some(Self(value)),
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Debug for ProviderSecret {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("ProviderSecret([REDACTED])")
    }
}

impl Drop for ProviderSecret {
    fn drop(&mut self) {
        self.0.zeroize();
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ProviderAccount {
    id: ProviderAccountId,
    provider_type: ProviderType,
    name: String,
    domains: Vec<DomainName>,
    enabled: bool,
    #[serde(default)]
    api_key: Option<ProviderSecret>,
    #[serde(default)]
    webhook_secret: Option<ProviderSecret>,
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
            api_key: None,
            webhook_secret: None,
        }
    }

    pub fn with_api_key(mut self, api_key: Option<String>) -> Self {
        self.set_api_key(api_key);
        self
    }

    pub fn with_webhook_secret(mut self, webhook_secret: Option<String>) -> Self {
        self.set_webhook_secret(webhook_secret);
        self
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

    pub fn api_key(&self) -> Option<&str> {
        self.api_key.as_ref().map(ProviderSecret::as_str)
    }

    pub fn api_key_configured(&self) -> bool {
        self.api_key.is_some()
    }

    pub fn webhook_secret(&self) -> Option<&str> {
        self.webhook_secret.as_ref().map(ProviderSecret::as_str)
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

    pub fn set_api_key(&mut self, api_key: Option<String>) {
        self.api_key = api_key.and_then(ProviderSecret::new);
    }

    pub fn replace_api_key(&mut self, api_key: String) {
        if let Some(secret) = ProviderSecret::new(api_key) {
            self.api_key = Some(secret);
        }
    }

    pub fn set_webhook_secret(&mut self, webhook_secret: Option<String>) {
        self.webhook_secret = webhook_secret.and_then(ProviderSecret::new);
    }

    pub fn replace_webhook_secret(&mut self, webhook_secret: String) {
        if let Some(secret) = ProviderSecret::new(webhook_secret) {
            self.webhook_secret = Some(secret);
        }
    }
}
