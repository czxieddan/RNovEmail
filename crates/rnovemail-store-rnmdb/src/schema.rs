pub const RECORD_CHUNK_NAMESPACE: &str = "__record_chunks";

pub const RNOVEMAIL_NAMESPACES: [&str; 14] = [
    "users_by_id",
    "users_by_email",
    "domains_by_id",
    "domains_by_name",
    "mailboxes_by_id",
    "mailboxes_by_email",
    "provider_accounts_by_id",
    "provider_accounts_by_domain",
    "outbound_messages_by_id",
    "inbound_messages_by_id",
    "webhook_events_by_provider_event",
    "api_tokens_by_prefix",
    "audit_events_by_time",
    RECORD_CHUNK_NAMESPACE,
];
