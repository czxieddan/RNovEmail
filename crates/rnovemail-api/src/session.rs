use std::{
    collections::HashMap,
    time::{Duration, SystemTime},
};

use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use rand::RngCore;

const SESSION_BYTES: usize = 32;
const IDLE_TTL: Duration = Duration::from_secs(30 * 60);
const ABSOLUTE_TTL: Duration = Duration::from_secs(8 * 60 * 60);
const FAILURE_LIMIT: u32 = 5;
const FAILURE_WINDOW: Duration = Duration::from_secs(15 * 60);
const LOCKOUT_TTL: Duration = Duration::from_secs(15 * 60);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SessionRole {
    Admin,
    User,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SessionPrincipal {
    pub role: SessionRole,
    pub subject: String,
}

#[derive(Debug)]
pub enum SessionError {
    Missing,
    Invalid,
    Locked,
}

#[derive(Default)]
pub struct SessionRegistry {
    sessions: HashMap<String, SessionRecord>,
    failures: HashMap<String, FailureRecord>,
}

struct SessionRecord {
    principal: SessionPrincipal,
    fingerprint: String,
    issued_at: SystemTime,
    last_seen: SystemTime,
}

struct FailureRecord {
    count: u32,
    first_seen: SystemTime,
    locked_until: Option<SystemTime>,
}

impl SessionRegistry {
    pub fn create(&mut self, role: SessionRole, subject: String, fingerprint: String) -> String {
        let id = random_session_id();
        let now = SystemTime::now();
        self.sessions.insert(
            id.clone(),
            SessionRecord {
                principal: SessionPrincipal { role, subject },
                fingerprint,
                issued_at: now,
                last_seen: now,
            },
        );
        id
    }

    pub fn validate(
        &mut self,
        session_id: &str,
        role: SessionRole,
        fingerprint: &str,
    ) -> Result<SessionPrincipal, SessionError> {
        let now = SystemTime::now();
        let Some(record) = self.sessions.get_mut(session_id) else {
            return Err(SessionError::Missing);
        };
        if record.expired(now) || record.fingerprint != fingerprint || record.principal.role != role
        {
            self.sessions.remove(session_id);
            return Err(SessionError::Invalid);
        }
        record.last_seen = now;
        Ok(record.principal.clone())
    }

    pub fn remove(&mut self, session_id: &str) {
        self.sessions.remove(session_id);
    }

    pub fn ensure_login_allowed(&mut self, key: &str) -> Result<(), SessionError> {
        let now = SystemTime::now();
        let Some(record) = self.failures.get(key) else {
            return Ok(());
        };
        match record.locked_until {
            Some(until) if until > now => Err(SessionError::Locked),
            _ => Ok(()),
        }
    }

    pub fn record_failure(&mut self, key: String) {
        let now = SystemTime::now();
        let record = self.failures.entry(key).or_insert_with(|| FailureRecord {
            count: 0,
            first_seen: now,
            locked_until: None,
        });
        if record.window_expired(now) {
            record.count = 0;
            record.first_seen = now;
            record.locked_until = None;
        }
        record.count += 1;
        if record.count >= FAILURE_LIMIT {
            record.locked_until = Some(now + LOCKOUT_TTL);
        }
    }

    pub fn record_success(&mut self, key: &str) {
        self.failures.remove(key);
    }
}

impl SessionRecord {
    fn expired(&self, now: SystemTime) -> bool {
        elapsed(self.last_seen, now) > IDLE_TTL || elapsed(self.issued_at, now) > ABSOLUTE_TTL
    }
}

impl FailureRecord {
    fn window_expired(&self, now: SystemTime) -> bool {
        elapsed(self.first_seen, now) > FAILURE_WINDOW
    }
}

fn elapsed(start: SystemTime, now: SystemTime) -> Duration {
    now.duration_since(start).unwrap_or(Duration::ZERO)
}

fn random_session_id() -> String {
    let mut bytes = [0_u8; SESSION_BYTES];
    rand::rngs::OsRng.fill_bytes(&mut bytes);
    URL_SAFE_NO_PAD.encode(bytes)
}
