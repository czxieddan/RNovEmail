mod errors;
mod forms;
mod i18n;
mod models;
mod pages;
mod theme;

pub use errors::AdminError;
pub use forms::{DomainForm, MailboxForm, ProviderAccountForm, UserForm};
pub use i18n::{Lang, Text, text};
pub use models::{
    AdminData, AdminSection, AuditRow, DomainRow, LoginScopeView, MailboxRow, MessageAttachmentRow,
    MessageDetailRow, MessageHeaderRow, MessageRow, PageContext, PortalData, PortalMessageData,
    ProviderRow, UserRow,
};
pub use pages::{admin_page, login_page, portal_message_page, portal_page};
pub use theme::Theme;
