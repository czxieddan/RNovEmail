use async_trait::async_trait;

use crate::StoreError;

#[async_trait]
pub trait UnitOfWork: Send + Sync {
    async fn commit(&self) -> Result<(), StoreError>;
    async fn rollback(&self) -> Result<(), StoreError>;
}
