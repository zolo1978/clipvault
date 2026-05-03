use crate::models::SensitiveKind;
use std::collections::HashMap;
use zeroize::Zeroize;

#[derive(Debug, Clone, PartialEq)]
pub enum EntryState {
    Available(Vec<u8>),
    Expired,
}

#[allow(dead_code)]
struct SensitiveEntry {
    content: Vec<u8>,
    kind: SensitiveKind,
    inserted_at: i64,
}

impl Drop for SensitiveEntry {
    fn drop(&mut self) {
        self.content.zeroize();
    }
}

pub struct SensitiveStore {
    entries: HashMap<String, SensitiveEntry>,
    ttl_ms: u64,
}

impl SensitiveStore {
    pub fn new(ttl_ms: u64) -> Self {
        Self {
            entries: HashMap::new(),
            ttl_ms,
        }
    }

    pub fn insert(&mut self, id: String, content: Vec<u8>, kind: SensitiveKind) {
        let now_ms = now_ms();
        self.entries.insert(
            id,
            SensitiveEntry {
                content,
                kind,
                inserted_at: now_ms,
            },
        );
    }

    pub fn get(&mut self, id: &str) -> Option<EntryState> {
        let ttl_ms = self.ttl_ms;
        if let Some(entry) = self.entries.get(id) {
            if now_ms() - entry.inserted_at > ttl_ms as i64 {
                self.entries.remove(id);
                return Some(EntryState::Expired);
            }
            return Some(EntryState::Available(entry.content.clone()));
        }
        None
    }

    pub fn is_expired(&self, id: &str) -> bool {
        if let Some(entry) = self.entries.get(id) {
            return now_ms() - entry.inserted_at > self.ttl_ms as i64;
        }
        true
    }

    pub fn cleanup_expired(&mut self) {
        let now = now_ms();
        let ttl = self.ttl_ms as i64;
        let expired: Vec<String> = self
            .entries
            .iter()
            .filter(|(_, e)| now - e.inserted_at > ttl)
            .map(|(id, _)| id.clone())
            .collect();
        for id in expired {
            self.entries.remove(&id);
        }
    }
}

fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64
}
