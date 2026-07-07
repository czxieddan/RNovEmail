use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum AuditActor {
    System,
    ApiToken(String),
    User(String),
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum AuditResult {
    Accepted,
    Rejected,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AuditEvent {
    pub actor: AuditActor,
    pub action: String,
    pub target: String,
    pub request_id: String,
    pub result: AuditResult,
    pub at: DateTime<Utc>,
}

impl AuditEvent {
    pub fn new(
        actor: AuditActor,
        action: impl Into<String>,
        target: impl Into<String>,
        request_id: impl Into<String>,
        result: AuditResult,
    ) -> Self {
        Self {
            actor,
            action: action.into(),
            target: target.into(),
            request_id: request_id.into(),
            result,
            at: Utc::now(),
        }
    }
}
