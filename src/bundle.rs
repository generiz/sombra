use blake3::Hasher;
use rand::random;
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
        let entropy: [u8; 16] = random();
        let mut hasher = Hasher::new();
        hasher.update(&created_at_ms.to_be_bytes());
        hasher.update(&expires_at_ms.to_be_bytes());
        hasher.update(&[hop_limit, priority_code(priority)]);
        hasher.update(&(payload.len() as u64).to_be_bytes());
        hasher.update(&entropy);
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

fn priority_code(priority: Priority) -> u8 {
    match priority {
        Priority::Routine => 0,
        Priority::Important => 1,
        Priority::Urgent => 2,
    }
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identical_payloads_get_distinct_bundle_ids() {
        let a = Bundle::new(b"same", Duration::from_secs(60), 4, Priority::Routine);
        let b = Bundle::new(b"same", Duration::from_secs(60), 4, Priority::Routine);
        assert_ne!(a.id, b.id);
    }

    #[test]
    fn forwarding_preserves_identity_and_decrements_hop_limit() {
        let bundle = Bundle::new(b"payload", Duration::from_secs(60), 2, Priority::Important);
        let forwarded = bundle.forwarded().unwrap();
        assert_eq!(forwarded.id, bundle.id);
        assert_eq!(forwarded.hop_limit, 1);
    }
}
