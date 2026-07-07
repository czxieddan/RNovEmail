use std::collections::HashSet;

use crate::WebhookEventKey;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReplayStatus {
    Accepted,
    Duplicate,
}

#[derive(Default)]
pub struct ReplayGuard {
    seen: HashSet<WebhookEventKey>,
}

impl ReplayGuard {
    pub fn remember(&mut self, key: WebhookEventKey) -> ReplayStatus {
        match self.seen.insert(key) {
            true => ReplayStatus::Accepted,
            false => ReplayStatus::Duplicate,
        }
    }
}
