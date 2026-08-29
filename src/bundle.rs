use blake3::Hasher;
use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub type BundleId = String;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Priority {
    Routine,
    Important,
    Urgent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bundle {
    pub id: BundleId,
    pub created_at_ms: u64,
    pub expires_at_ms: u64,
    pub hop_limit: u8,
    pub priority: Priority,
    pub payload_bytes: usize,
}

impl Bundle {
    pub fn new(payload: &[u8], lifetime: Duration, hop_limit: u8, priority: Priority) -> Self {
        let created_at_ms = now_ms();
        let expires_at_ms = created_at_ms.saturating_add(lifetime.as_millis() as u64);
        let mut hasher = Hasher::new();
        hasher.update(&created_at_ms.to_be_bytes());
        hasher.update(&(payload.len() as u64).to_be_bytes());
        hasher.update(payload);
        let id = hasher.finalize().to_hex()[..24].to_string();

        Self {
            id,
            created_at_ms,
            expires_at_ms,
            hop_limit,
            priority,
            payload_bytes: payload.len(),
        }
    }

    pub fn is_expired_at(&self, now_ms: u64) -> bool {
        now_ms >= self.expires_at_ms
    }

    pub fn can_forward(&self) -> bool {
        self.hop_limit > 0
    }

    pub fn forwarded(&self) -> Option<Self> {
        if !self.can_forward() {
            return None;
        }
        let mut next = self.clone();
        next.hop_limit -= 1;
        Some(next)
    }
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}
