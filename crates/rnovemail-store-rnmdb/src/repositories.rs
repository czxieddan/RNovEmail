use std::{
    fmt::Write,
    path::{Path, PathBuf},
    sync::Mutex,
};

use crate::schema::{RECORD_CHUNK_NAMESPACE, RNOVEMAIL_NAMESPACES};
use crate::{MigrationPlan, keys::namespace_key};
use async_trait::async_trait;
use rnmdb_common::ids::PageId;
use rnmdb_storage::{
    Page, PageCryptoKey, PageSize, SingleFileBackend, SingleFileOptions, StorageBackend, SyncStatus,
};
use rnovemail_domain::{
    AuditEvent, DomainName, EmailAddress, InboundMessage, Mailbox, MessageDirection,
    MessageUserState, OutboundMessage, ProviderAccount, User,
};
use rnovemail_store::{
    AuditRepository, DomainRepository, MailboxRepository, MessageRepository, ProviderRepository,
    StoreError, TokenRepository, UserRepository, WebhookRepository,
};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use sha2::{Digest, Sha256};
use zeroize::Zeroize;

const DATABASE_FILE: &str = "rnovemail.rnmdb";
const PAGE_SIZE_BYTES: usize = PageSize::DEFAULT_BYTES;
const RECORD_HEADER_BYTES: usize = 4;
const CHUNK_VALUE_BYTES: usize = 512;
const META_PAGE_ID: u64 = 1;
const FIRST_RECORD_PAGE_ID: u64 = 2;

#[derive(Clone, Eq, PartialEq)]
pub struct RnovStoreKey([u8; 32]);

impl RnovStoreKey {
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    pub fn derive_from_master_key(material: &[u8]) -> Result<Self, StoreError> {
        reject_empty_key_material(material)?;
        let digest = page_key_digest(material);
        let mut key = [0_u8; 32];
        key.copy_from_slice(&digest);
        Ok(Self(key))
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

fn reject_empty_key_material(material: &[u8]) -> Result<(), StoreError> {
    match material.iter().all(|byte| byte.is_ascii_whitespace()) {
        true => Err(StoreError::OperationFailed),
        false => Ok(()),
    }
}

fn page_key_digest(material: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(b"rnovemail:rnmdb:v1:page-key");
    hasher.update(material);
    hasher.finalize().into()
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
        let records = records_for_value(namespace, key, value);
        let backend = self.backend()?;
        let mut meta = load_meta(&backend)?;
        for record in records {
            meta = write_record(&backend, meta, &record)?;
        }
        backend
            .sync()
            .map(|_| ())
            .map_err(|_| StoreError::OperationFailed)
    }

    pub fn delete_raw(&self, namespace: &str, key: &str) -> Result<(), StoreError> {
        ensure_namespace(namespace)?;
        let record = RawRecord::new(namespace, key, &[]);
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
        match record {
            Some(record) => record_value(&backend, &meta, record),
            None => Ok(None),
        }
    }

    pub fn list_raw(&self, namespace: &str) -> Result<Vec<(String, Vec<u8>)>, StoreError> {
        ensure_namespace(namespace)?;
        let backend = self.backend()?;
        let meta = load_meta(&backend)?;
        let records = list_records(&backend, &meta, namespace)?;
        active_records(&backend, &meta, records)
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

#[async_trait]
impl UserRepository for RnovStore {
    async fn put_user(&self, user: User) -> Result<(), StoreError> {
        put_typed(self, "users_by_email", user.primary_email().as_str(), &user)
    }

    async fn get_user_by_email(&self, email: &EmailAddress) -> Result<User, StoreError> {
        get_required(self, "users_by_email", email.as_str())
    }

    async fn list_users(&self) -> Result<Vec<User>, StoreError> {
        list_typed(self, "users_by_email")
    }
}

#[async_trait]
impl DomainRepository for RnovStore {
    async fn put_domain(&self, domain: DomainName) -> Result<(), StoreError> {
        put_typed(self, "domains_by_name", domain.as_str(), &domain)
    }

    async fn delete_domain(&self, domain: &DomainName) -> Result<(), StoreError> {
        self.delete_raw("domains_by_name", domain.as_str())
    }

    async fn contains_domain(&self, domain: &DomainName) -> Result<bool, StoreError> {
        self.get_raw("domains_by_name", domain.as_str())
            .map(|record| record.is_some())
    }

    async fn list_domains(&self) -> Result<Vec<DomainName>, StoreError> {
        list_typed(self, "domains_by_name")
    }
}

#[async_trait]
impl MailboxRepository for RnovStore {
    async fn put_mailbox(&self, mailbox: Mailbox) -> Result<(), StoreError> {
        put_typed(
            self,
            "mailboxes_by_email",
            mailbox.address().as_str(),
            &mailbox,
        )
    }

    async fn get_mailbox_by_email(&self, email: &EmailAddress) -> Result<Mailbox, StoreError> {
        get_required(self, "mailboxes_by_email", email.as_str())
    }

    async fn list_mailboxes(&self) -> Result<Vec<Mailbox>, StoreError> {
        list_typed(self, "mailboxes_by_email")
    }
}

#[async_trait]
impl ProviderRepository for RnovStore {
    async fn put_provider(&self, provider: ProviderAccount) -> Result<(), StoreError> {
        put_typed(
            self,
            "provider_accounts_by_id",
            &json_key(&provider.id())?,
            &provider,
        )
    }

    async fn delete_provider(&self, provider: &ProviderAccount) -> Result<(), StoreError> {
        self.delete_raw("provider_accounts_by_id", &json_key(&provider.id())?)
    }

    async fn list_providers(&self) -> Result<Vec<ProviderAccount>, StoreError> {
        list_typed(self, "provider_accounts_by_id")
    }
}

#[async_trait]
impl MessageRepository for RnovStore {
    async fn put_outbound(&self, message: OutboundMessage) -> Result<(), StoreError> {
        put_typed(
            self,
            "outbound_messages_by_id",
            &json_key(&message.id)?,
            &message,
        )
    }

    async fn put_inbound(&self, message: InboundMessage) -> Result<(), StoreError> {
        put_typed(
            self,
            "inbound_messages_by_id",
            &json_key(&message.id)?,
            &message,
        )
    }

    async fn put_message_user_state(&self, state: MessageUserState) -> Result<(), StoreError> {
        put_typed(
            self,
            "message_user_states_by_key",
            &message_user_state_key(&state),
            &state,
        )
    }

    async fn list_outbound(&self) -> Result<Vec<OutboundMessage>, StoreError> {
        list_typed(self, "outbound_messages_by_id")
    }

    async fn list_inbound(&self) -> Result<Vec<InboundMessage>, StoreError> {
        list_typed(self, "inbound_messages_by_id")
    }

    async fn list_message_user_states(&self) -> Result<Vec<MessageUserState>, StoreError> {
        list_typed(self, "message_user_states_by_key")
    }
}

#[async_trait]
impl WebhookRepository for RnovStore {
    async fn remember_event(&self, provider: &str, event_id: &str) -> Result<bool, StoreError> {
        let key = namespace_key(provider, event_id);
        match self.get_raw("webhook_events_by_provider_event", &key)? {
            Some(_) => Ok(false),
            None => put_typed(self, "webhook_events_by_provider_event", &key, &true).map(|_| true),
        }
    }
}

#[async_trait]
impl TokenRepository for RnovStore {
    async fn put_token_hash(&self, prefix: String, hash: String) -> Result<(), StoreError> {
        put_typed(self, "api_tokens_by_prefix", &prefix, &hash)
    }

    async fn get_token_hash(&self, prefix: &str) -> Result<String, StoreError> {
        get_required(self, "api_tokens_by_prefix", prefix)
    }
}

#[async_trait]
impl AuditRepository for RnovStore {
    async fn append_audit(&self, event: AuditEvent) -> Result<(), StoreError> {
        let key = audit_key(&event);
        put_typed(self, "audit_events_by_time", &key, &event)
    }

    async fn list_audit(&self) -> Result<Vec<AuditEvent>, StoreError> {
        list_typed(self, "audit_events_by_time")
    }
}

#[derive(Clone, Deserialize, Serialize)]
struct RawRecord {
    namespace: String,
    key: String,
    value: Vec<u8>,
    #[serde(default)]
    chunk_count: usize,
    #[serde(default)]
    value_len: usize,
}

impl RawRecord {
    fn new(namespace: &str, key: &str, value: &[u8]) -> Self {
        Self {
            namespace: namespace.to_string(),
            key: key.to_string(),
            value: value.to_vec(),
            chunk_count: 0,
            value_len: value.len(),
        }
    }

    fn chunked(namespace: &str, key: &str, value_len: usize, chunk_count: usize) -> Self {
        Self {
            namespace: namespace.to_string(),
            key: key.to_string(),
            value: Vec::new(),
            chunk_count,
            value_len,
        }
    }

    fn deleted(&self) -> bool {
        self.value.is_empty() && self.chunk_count == 0
    }

    fn chunked_value(&self) -> bool {
        self.chunk_count > 0
    }
}

fn records_for_value(namespace: &str, key: &str, value: &[u8]) -> Vec<RawRecord> {
    let inline = RawRecord::new(namespace, key, value);
    match record_fits(&inline) {
        true => vec![inline],
        false => chunked_records(namespace, key, value),
    }
}

fn chunked_records(namespace: &str, key: &str, value: &[u8]) -> Vec<RawRecord> {
    let mut records = value_chunks(namespace, key, value);
    records.push(RawRecord::chunked(
        namespace,
        key,
        value.len(),
        records.len(),
    ));
    records
}

fn value_chunks(namespace: &str, key: &str, value: &[u8]) -> Vec<RawRecord> {
    value
        .chunks(CHUNK_VALUE_BYTES)
        .enumerate()
        .map(|(index, chunk)| {
            RawRecord::new(
                RECORD_CHUNK_NAMESPACE,
                &chunk_key(namespace, key, index),
                chunk,
            )
        })
        .collect()
}

fn record_fits(record: &RawRecord) -> bool {
    serde_json::to_vec(record)
        .map(|encoded| encoded.len() <= PAGE_SIZE_BYTES - RECORD_HEADER_BYTES)
        .unwrap_or(false)
}

fn chunk_key(namespace: &str, key: &str, index: usize) -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"rnovemail:rnmdb:chunk:v1");
    hasher.update(namespace.as_bytes());
    hasher.update([0]);
    hasher.update(key.as_bytes());
    hasher.update([0]);
    hasher.update(index.to_be_bytes());
    digest_hex(&hasher.finalize())
}

fn digest_hex(bytes: &[u8]) -> String {
    let mut value = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        let _ = write!(value, "{byte:02x}");
    }
    value
}

#[derive(Clone, Deserialize, Serialize)]
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
) -> Result<StoreMeta, StoreError> {
    match find_record_page(backend, &meta, &record.namespace, &record.key)? {
        Some(page_id) => {
            write_raw_page(backend, page_id, record)?;
            Ok(meta)
        }
        None => insert_record(backend, &mut meta, record),
    }
}

fn insert_record(
    backend: &SingleFileBackend,
    meta: &mut StoreMeta,
    record: &RawRecord,
) -> Result<StoreMeta, StoreError> {
    let page_id = meta.allocate_page()?;
    write_raw_page(backend, page_id, record)?;
    write_meta_page(backend, meta)?;
    Ok(meta.clone())
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

fn list_records(
    backend: &SingleFileBackend,
    meta: &StoreMeta,
    namespace: &str,
) -> Result<Vec<RawRecord>, StoreError> {
    let mut records = Vec::new();
    for page_id in meta.record_page_ids() {
        append_matching_record(backend, page_id, namespace, &mut records)?;
    }
    Ok(records)
}

fn append_matching_record(
    backend: &SingleFileBackend,
    page_id: PageId,
    namespace: &str,
    records: &mut Vec<RawRecord>,
) -> Result<(), StoreError> {
    let Some(record) = maybe_read_raw_page(backend, page_id)? else {
        return Ok(());
    };
    if record.namespace == namespace {
        records.push(record);
    }
    Ok(())
}

fn active_records(
    backend: &SingleFileBackend,
    meta: &StoreMeta,
    records: Vec<RawRecord>,
) -> Result<Vec<(String, Vec<u8>)>, StoreError> {
    let mut active = Vec::new();
    for record in records {
        if let Some(value) = record_value(backend, meta, record.clone())? {
            active.push((record.key, value));
        }
    }
    Ok(active)
}

fn record_value(
    backend: &SingleFileBackend,
    meta: &StoreMeta,
    record: RawRecord,
) -> Result<Option<Vec<u8>>, StoreError> {
    if record.deleted() {
        return Ok(None);
    }
    match record.chunked_value() {
        true => read_chunked_value(backend, meta, &record).map(Some),
        false => Ok(Some(record.value)),
    }
}

fn read_chunked_value(
    backend: &SingleFileBackend,
    meta: &StoreMeta,
    record: &RawRecord,
) -> Result<Vec<u8>, StoreError> {
    let mut value = Vec::with_capacity(record.value_len);
    for index in 0..record.chunk_count {
        value.extend(read_value_chunk(backend, meta, record, index)?);
    }
    ensure_chunked_len(value.len(), record.value_len)?;
    Ok(value)
}

fn read_value_chunk(
    backend: &SingleFileBackend,
    meta: &StoreMeta,
    record: &RawRecord,
    index: usize,
) -> Result<Vec<u8>, StoreError> {
    let key = chunk_key(&record.namespace, &record.key, index);
    let chunk = find_record(backend, meta, RECORD_CHUNK_NAMESPACE, &key)?
        .ok_or(StoreError::OperationFailed)?;
    match chunk.chunked_value() || chunk.deleted() {
        true => Err(StoreError::OperationFailed),
        false => Ok(chunk.value),
    }
}

fn ensure_chunked_len(actual: usize, expected: usize) -> Result<(), StoreError> {
    match actual == expected {
        true => Ok(()),
        false => Err(StoreError::OperationFailed),
    }
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

fn put_typed<T: Serialize>(
    store: &RnovStore,
    namespace: &str,
    key: &str,
    value: &T,
) -> Result<(), StoreError> {
    let encoded = serde_json::to_vec(value).map_err(|_| StoreError::OperationFailed)?;
    store.put_raw(namespace, key, &encoded)
}

fn get_required<T: DeserializeOwned>(
    store: &RnovStore,
    namespace: &str,
    key: &str,
) -> Result<T, StoreError> {
    let Some(record) = store.get_raw(namespace, key)? else {
        return Err(StoreError::NotFound);
    };
    serde_json::from_slice(&record).map_err(|_| StoreError::OperationFailed)
}

fn list_typed<T: DeserializeOwned>(
    store: &RnovStore,
    namespace: &str,
) -> Result<Vec<T>, StoreError> {
    store
        .list_raw(namespace)?
        .into_iter()
        .map(|(_, value)| serde_json::from_slice(&value).map_err(|_| StoreError::OperationFailed))
        .collect()
}

fn json_key<T: Serialize>(value: &T) -> Result<String, StoreError> {
    serde_json::to_string(value).map_err(|_| StoreError::OperationFailed)
}

fn audit_key(event: &AuditEvent) -> String {
    format!("{}:{}", event.at.to_rfc3339(), event.request_id)
}

fn message_user_state_key(state: &MessageUserState) -> String {
    format!(
        "{}:{}:{}",
        state.user_email.as_str(),
        direction_name(state.direction),
        state.provider_message_id
    )
}

fn direction_name(direction: MessageDirection) -> &'static str {
    match direction {
        MessageDirection::Inbound => "inbound",
        MessageDirection::Outbound => "outbound",
    }
}
