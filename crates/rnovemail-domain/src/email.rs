use serde::{Deserialize, Serialize};

use crate::{DomainError, DomainName};

#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct EmailAddress {
    value: String,
    domain: DomainName,
}

impl EmailAddress {
    pub fn parse(input: impl AsRef<str>) -> Result<Self, DomainError> {
        let trimmed = input.as_ref().trim();
        reject_display_syntax(trimmed)?;
        reject_email_spaces(trimmed)?;
        let (local, domain) = split_mailbox(trimmed)?;
        reject_local(local)?;
        let domain = DomainName::parse(domain)?;
        let value = format!("{}@{}", local.to_ascii_lowercase(), domain.as_str());
        Ok(Self { value, domain })
    }

    pub fn as_str(&self) -> &str {
        &self.value
    }

    pub fn domain(&self) -> &DomainName {
        &self.domain
    }
}

fn reject_display_syntax(value: &str) -> Result<(), DomainError> {
    match value.contains('<') || value.contains('>') {
        true => Err(DomainError::DisplaySyntax),
        false => Ok(()),
    }
}

fn reject_email_spaces(value: &str) -> Result<(), DomainError> {
    match value.chars().any(char::is_whitespace) {
        true => Err(DomainError::Whitespace),
        false => Ok(()),
    }
}

fn split_mailbox(value: &str) -> Result<(&str, &str), DomainError> {
    let mut parts = value.split('@');
    let local = parts.next().ok_or(DomainError::EmailShape)?;
    let domain = parts.next().ok_or(DomainError::EmailShape)?;
    match parts.next().is_none() {
        true => Ok((local, domain)),
        false => Err(DomainError::EmailShape),
    }
}

fn reject_local(value: &str) -> Result<(), DomainError> {
    match value.is_empty() || value.len() > 64 {
        true => Err(DomainError::EmailShape),
        false => Ok(()),
    }
}
