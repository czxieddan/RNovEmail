#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UserForm {
    pub display_name: String,
    pub email: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DomainForm {
    pub domain: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProviderAccountForm {
    pub name: String,
    pub provider: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MailboxForm {
    pub owner_email: String,
    pub mailbox_email: String,
}
