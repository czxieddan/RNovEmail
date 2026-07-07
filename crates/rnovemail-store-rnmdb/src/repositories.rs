use std::{
    path::{Path, PathBuf},
    sync::Mutex,
};

use crate::MigrationPlan;
use crate::schema::RNOVEMAIL_NAMESPACES;
use rnmdb_common::ids::PageId;
use rnmdb_storage::{
    Page, PageCryptoKey, PageSize, SingleFileBackend, SingleFileOptions, StorageBackend, SyncStatus,
};
use rnovemail_store::StoreError;
use serde::{Deserialize, Serialize};
use zeroize::Zeroize;

const DATABASE_FILE: &str = "rnovemail.rnmdb";
const PAGE_SIZE_BYTES: usize = PageSize::DEFAULT_BYTES;
const RECORD_HEADER_BYTES: usize = 4;
const META_PAGE_ID: u64 = 1;
const FIRST_RECORD_PAGE_ID: u64 = 2;

#[derive(Clone, Eq, PartialEq)]
pub struct RnovStoreKey([u8; 32]);

impl RnovStoreKey {
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    fn page_crypto_key(mut self) -> PageCryptoKey {
        let page_key = PageCryptoKey::from_bytes(self.0);
        self.0.zeroize();
        page_key
    }
}

impl Drop for RnovStoreKey {
    fn drop(&mut self) {
        self.0.zeroize();
    }
}

impl std::fmt::Debug for RnovStoreKey {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("RnovStoreKey(..)")
    }
}

impl From<[u8; 32]> for RnovStoreKey {
    fn from(bytes: [u8; 32]) -> Self {
        Self::from_bytes(bytes)
    }
}

pub struct RnovStore {
    data_dir: PathBuf,
    backend: Mutex<SingleFileBackend>,
    migrations: MigrationPlan,
}

impl RnovStore {
    pub fn open(data_dir: impl AsRef<Path>, key: RnovStoreKey) -> Result<Self, StoreError> {
        let data_dir = data_dir.as_ref().to_path_buf();
        std::fs::create_dir_all(&data_dir).map_err(|_| StoreError::OperationFailed)?;
        let database_path = data_dir.join(DATABASE_FILE);
        let backend = open_backend(&database_path, key.page_crypto_key())?;
        Ok(Self {
            data_dir,
            backend: Mutex::new(backend),
            migrations: MigrationPlan::current(),
        })
    }

    pub fn data_dir(&self) -> &Path {
        &self.data_dir
    }

    pub fn database_path(&self) -> PathBuf {
        self.data_dir.join(DATABASE_FILE)
    }

    pub fn migrations(&self) -> &MigrationPlan {
        &self.migrations
    }

    pub fn put_raw(&self, namespace: &str, key: &str, value: &[u8]) -> Result<(), StoreError> {
        ensure_namespace(namespace)?;
        let record = RawRecord::new(namespace, key, value);
        let backend = self.backend()?;
        let meta = load_meta(&backend)?;
        write_record(&backend, meta, &record)?;
        backend
            .sync()
            .map(|_| ())
            .map_err(|_| StoreError::OperationFailed)
    }

    pub fn get_raw(&self, namespace: &str, key: &str) -> Result<Option<Vec<u8>>, StoreError> {
        ensure_namespace(namespace)?;
        let backend = self.backend()?;
        let meta = load_meta(&backend)?;
        let record = find_record(&backend, &meta, namespace, key)?;
        Ok(record.map(|record| record.value))
    }

    pub fn sync_status(&self) -> Result<SyncStatus, StoreError> {
        self.backend()?
            .sync()
            .map_err(|_| StoreError::OperationFailed)
    }

    fn backend(&self) -> Result<std::sync::MutexGuard<'_, SingleFileBackend>, StoreError> {
        self.backend.lock().map_err(|_| StoreError::OperationFailed)
    }
}

#[derive(Deserialize, Serialize)]
struct RawRecord {
    namespace: String,
    key: String,
    value: Vec<u8>,
}

impl RawRecord {
    fn new(namespace: &str, key: &str, value: &[u8]) -> Self {
        Self {
            namespace: namespace.to_string(),
            key: key.to_string(),
            value: value.to_vec(),
        }
    }
}

#[derive(Deserialize, Serialize)]
struct StoreMeta {
    next_page_id: u64,
}

impl StoreMeta {
    fn allocate_page(&mut self) -> Result<PageId, StoreError> {
        let page_id = PageId::new(self.next_page_id);
        self.next_page_id = self
            .next_page_id
            .checked_add(1)
            .ok_or(StoreError::OperationFailed)?;
        Ok(page_id)
    }

    fn record_page_ids(&self) -> impl Iterator<Item = PageId> {
        (FIRST_RECORD_PAGE_ID..self.next_page_id).map(PageId::new)
    }
}

impl Default for StoreMeta {
    fn default() -> Self {
        Self {
            next_page_id: FIRST_RECORD_PAGE_ID,
        }
    }
}

fn open_backend(path: &Path, key: PageCryptoKey) -> Result<SingleFileBackend, StoreError> {
    match path.exists() {
        true => {
            SingleFileBackend::open_with_key(path, key).map_err(|_| StoreError::OperationFailed)
        }
        false => create_backend(path, key),
    }
}

fn create_backend(path: &Path, key: PageCryptoKey) -> Result<SingleFileBackend, StoreError> {
    let options = SingleFileOptions::new(PageSize::new(PAGE_SIZE_BYTES)).with_page_key(key);
    SingleFileBackend::create(path, options).map_err(|_| StoreError::OperationFailed)
}

fn ensure_namespace(namespace: &str) -> Result<(), StoreError> {
    match RNOVEMAIL_NAMESPACES.contains(&namespace) {
        true => Ok(()),
        false => Err(StoreError::OperationFailed),
    }
}

fn load_meta(backend: &SingleFileBackend) -> Result<StoreMeta, StoreError> {
    let page = read_payload_page(backend, PageId::new(META_PAGE_ID))?;
    match page {
        Some(payload) => unpack_meta(&payload),
        None => Ok(StoreMeta::default()),
    }
}

fn write_record(
    backend: &SingleFileBackend,
    mut meta: StoreMeta,
    record: &RawRecord,
) -> Result<(), StoreError> {
    match find_record_page(backend, &meta, &record.namespace, &record.key)? {
        Some(page_id) => write_raw_page(backend, page_id, record),
        None => insert_record(backend, &mut meta, record),
    }
}

fn insert_record(
    backend: &SingleFileBackend,
    meta: &mut StoreMeta,
    record: &RawRecord,
) -> Result<(), StoreError> {
    let page_id = meta.allocate_page()?;
    write_raw_page(backend, page_id, record)?;
    write_meta_page(backend, meta)
}

fn find_record(
    backend: &SingleFileBackend,
    meta: &StoreMeta,
    namespace: &str,
    key: &str,
) -> Result<Option<RawRecord>, StoreError> {
    let Some(page_id) = find_record_page(backend, meta, namespace, key)? else {
        return Ok(None);
    };
    read_raw_page(backend, page_id).map(Some)
}

fn find_record_page(
    backend: &SingleFileBackend,
    meta: &StoreMeta,
    namespace: &str,
    key: &str,
) -> Result<Option<PageId>, StoreError> {
    for page_id in meta.record_page_ids() {
        let Some(record) = maybe_read_raw_page(backend, page_id)? else {
            continue;
        };
        if record.namespace == namespace && record.key == key {
            return Ok(Some(page_id));
        }
    }
    Ok(None)
}

fn write_raw_page(
    backend: &SingleFileBackend,
    page_id: PageId,
    record: &RawRecord,
) -> Result<(), StoreError> {
    let payload = encode_record(record)?;
    write_payload_page(backend, page_id, payload)
}

fn write_meta_page(backend: &SingleFileBackend, meta: &StoreMeta) -> Result<(), StoreError> {
    let payload = encode_meta(meta)?;
    write_payload_page(backend, PageId::new(META_PAGE_ID), payload)
}

fn read_raw_page(backend: &SingleFileBackend, page_id: PageId) -> Result<RawRecord, StoreError> {
    let payload = read_payload_page(backend, page_id)?.ok_or(StoreError::OperationFailed)?;
    unpack_record(&payload)
}

fn maybe_read_raw_page(
    backend: &SingleFileBackend,
    page_id: PageId,
) -> Result<Option<RawRecord>, StoreError> {
    read_payload_page(backend, page_id)?
        .map(|payload| unpack_record(&payload))
        .transpose()
}

fn read_payload_page(
    backend: &SingleFileBackend,
    page_id: PageId,
) -> Result<Option<Vec<u8>>, StoreError> {
    backend
        .read_page(page_id)
        .map(|page| page.map(|page| page.into_payload()))
        .map_err(|_| StoreError::OperationFailed)
}

fn write_payload_page(
    backend: &SingleFileBackend,
    page_id: PageId,
    payload: Vec<u8>,
) -> Result<(), StoreError> {
    let page = Page::new(page_id, payload).map_err(|_| StoreError::OperationFailed)?;
    backend
        .write_page(page)
        .map_err(|_| StoreError::OperationFailed)
}

fn encode_record(record: &RawRecord) -> Result<Vec<u8>, StoreError> {
    let encoded = serde_json::to_vec(record).map_err(|_| StoreError::OperationFailed)?;
    pack_payload(encoded)
}

fn encode_meta(meta: &StoreMeta) -> Result<Vec<u8>, StoreError> {
    let encoded = serde_json::to_vec(meta).map_err(|_| StoreError::OperationFailed)?;
    pack_payload(encoded)
}

fn pack_payload(encoded: Vec<u8>) -> Result<Vec<u8>, StoreError> {
    ensure_record_fits(encoded.len())?;
    let mut payload = vec![0_u8; PAGE_SIZE_BYTES];
    payload[..RECORD_HEADER_BYTES].copy_from_slice(&(encoded.len() as u32).to_be_bytes());
    payload[RECORD_HEADER_BYTES..RECORD_HEADER_BYTES + encoded.len()].copy_from_slice(&encoded);
    Ok(payload)
}

fn ensure_record_fits(len: usize) -> Result<(), StoreError> {
    match len <= PAGE_SIZE_BYTES - RECORD_HEADER_BYTES {
        true => Ok(()),
        false => Err(StoreError::OperationFailed),
    }
}

fn unpack_record(payload: &[u8]) -> Result<RawRecord, StoreError> {
    let body = unpack_payload(payload)?;
    serde_json::from_slice(body).map_err(|_| StoreError::OperationFailed)
}

fn unpack_meta(payload: &[u8]) -> Result<StoreMeta, StoreError> {
    let body = unpack_payload(payload)?;
    serde_json::from_slice(body).map_err(|_| StoreError::OperationFailed)
}

fn unpack_payload(payload: &[u8]) -> Result<&[u8], StoreError> {
    let len = record_len(payload)?;
    payload
        .get(RECORD_HEADER_BYTES..RECORD_HEADER_BYTES + len)
        .ok_or(StoreError::OperationFailed)
}

fn record_len(payload: &[u8]) -> Result<usize, StoreError> {
    let bytes = payload
        .get(..RECORD_HEADER_BYTES)
        .ok_or(StoreError::OperationFailed)?;
    Ok(u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) as usize)
}
