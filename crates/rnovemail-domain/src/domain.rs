use serde::{Deserialize, Serialize};

use crate::DomainError;

#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct DomainName(String);

impl DomainName {
    pub fn parse(input: impl AsRef<str>) -> Result<Self, DomainError> {
        let trimmed = input.as_ref().trim();
        reject_domain_shape(trimmed)?;
        let ascii = idna::domain_to_ascii(trimmed).map_err(|_| DomainError::InvalidDomain)?;
        let normalized = ascii.to_ascii_lowercase();
        reject_domain_shape(&normalized)?;
        Ok(Self(normalized))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

fn reject_domain_shape(value: &str) -> Result<(), DomainError> {
    reject_empty(value)?;
    reject_mailbox(value)?;
    reject_spaces(value)?;
    reject_dot_edges(value)?;
    reject_dot_runs(value)
}

fn reject_empty(value: &str) -> Result<(), DomainError> {
    match value.is_empty() {
        true => Err(DomainError::Empty),
        false => Ok(()),
    }
}

fn reject_mailbox(value: &str) -> Result<(), DomainError> {
    match value.contains('@') {
        true => Err(DomainError::InvalidDomain),
        false => Ok(()),
    }
}

fn reject_spaces(value: &str) -> Result<(), DomainError> {
    match value.chars().any(char::is_whitespace) {
        true => Err(DomainError::Whitespace),
        false => Ok(()),
    }
}

fn reject_dot_edges(value: &str) -> Result<(), DomainError> {
    match value.starts_with('.') || value.ends_with('.') {
        true => Err(DomainError::InvalidDomain),
        false => Ok(()),
    }
}

fn reject_dot_runs(value: &str) -> Result<(), DomainError> {
    match value.contains("..") {
        true => Err(DomainError::InvalidDomain),
        false => Ok(()),
    }
}
