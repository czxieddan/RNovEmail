use thiserror::Error;

#[derive(Debug, Error)]
pub enum StoreError {
    #[error("record was not found")]
    NotFound,
    #[error("record already exists")]
    Conflict,
    #[error("storage operation failed")]
    OperationFailed,
}
