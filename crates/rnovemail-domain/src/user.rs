use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{EmailAddress, new_uuid};

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct UserId(Uuid);

impl UserId {
    pub fn new() -> Self {
        Self(new_uuid())
    }
}

impl Default for UserId {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum UserStatus {
    Active,
    Disabled,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum UserRole {
    Admin,
    MailUser,
    Auditor,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct User {
    id: UserId,
    display_name: String,
    primary_email: EmailAddress,
    roles: Vec<UserRole>,
    status: UserStatus,
    #[serde(default)]
    login_secret_hash: Option<String>,
}

impl User {
    pub fn assign(
        display_name: impl Into<String>,
        primary_email: EmailAddress,
        roles: impl IntoIterator<Item = UserRole>,
    ) -> Self {
        Self {
            id: UserId::new(),
            display_name: display_name.into(),
            primary_email,
            roles: roles.into_iter().collect(),
            status: UserStatus::Active,
            login_secret_hash: None,
        }
    }

    pub fn with_login_secret_hash(mut self, hash: Option<String>) -> Self {
        self.login_secret_hash = hash;
        self
    }

    pub fn id(&self) -> UserId {
        self.id
    }

    pub fn display_name(&self) -> &str {
        &self.display_name
    }

    pub fn primary_email(&self) -> &EmailAddress {
        &self.primary_email
    }

    pub fn roles(&self) -> &[UserRole] {
        &self.roles
    }

    pub fn status(&self) -> UserStatus {
        self.status
    }

    pub fn login_secret_hash(&self) -> Option<&str> {
        self.login_secret_hash.as_deref()
    }

    pub fn set_display_name(&mut self, display_name: String) {
        self.display_name = display_name;
    }

    pub fn set_roles(&mut self, roles: Vec<UserRole>) {
        self.roles = roles;
    }

    pub fn set_status(&mut self, status: UserStatus) {
        self.status = status;
    }

    pub fn set_login_secret_hash(&mut self, hash: Option<String>) {
        self.login_secret_hash = hash;
    }

    pub fn has_role(&self, role: UserRole) -> bool {
        self.roles.contains(&role)
    }
}
