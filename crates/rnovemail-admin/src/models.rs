use crate::{Lang, Theme};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PageContext {
    pub lang: Lang,
    pub theme: Theme,
    pub next: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LoginScopeView {
    Admin,
    User,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AdminSection {
    Audit,
    Dashboard,
    Domains,
    Mailboxes,
    Providers,
    Users,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct AdminData {
    pub users: Vec<UserRow>,
    pub domains: Vec<DomainRow>,
    pub providers: Vec<ProviderRow>,
    pub mailboxes: Vec<MailboxRow>,
    pub audit_events: Vec<AuditRow>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UserRow {
    pub display_name: String,
    pub email: String,
    pub roles: String,
    pub status: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DomainRow {
    pub domain: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProviderRow {
    pub id: String,
    pub name: String,
    pub provider_type: String,
    pub domains: String,
    pub enabled: bool,
    pub api_key_configured: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MailboxRow {
    pub owner: String,
    pub email: String,
    pub status: String,
    pub inbound_enabled: bool,
    pub outbound_enabled: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuditRow {
    pub at: String,
    pub action: String,
    pub target: String,
    pub result: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PortalData {
    pub email: String,
    pub mailboxes: Vec<MailboxRow>,
}
