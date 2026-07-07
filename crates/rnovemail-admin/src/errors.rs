use thiserror::Error;

#[derive(Debug, Error)]
pub enum AdminError {
    #[error("admin session is required")]
    MissingSession,
}
