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
        }
    }

    pub fn id(&self) -> UserId {
        self.id
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
}
