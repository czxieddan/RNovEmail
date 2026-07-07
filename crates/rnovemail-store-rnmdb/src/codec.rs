use serde::{Serialize, de::DeserializeOwned};

use rnovemail_store::StoreError;

pub fn encode<T: Serialize>(value: &T) -> Result<Vec<u8>, StoreError> {
    serde_json::to_vec(value).map_err(|_| StoreError::OperationFailed)
}

pub fn decode<T: DeserializeOwned>(value: &[u8]) -> Result<T, StoreError> {
    serde_json::from_slice(value).map_err(|_| StoreError::OperationFailed)
}
