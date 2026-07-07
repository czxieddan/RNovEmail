mod errors;
mod forms;
mod pages;

pub use errors::AdminError;
pub use forms::{DomainForm, MailboxForm, ProviderAccountForm, UserForm};
pub use pages::{
    audit_page, dashboard_page, domains_page, login_page, mailboxes_page, providers_page,
    users_page,
};
