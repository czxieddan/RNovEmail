mod errors;
mod forms;
mod pages;

pub use errors::AdminError;
pub use forms::{DomainForm, MailboxForm, ProviderAccountForm, UserForm};
pub use pages::{dashboard_page, login_page};
