use crate::scheduler::{BundleScheduler, ScheduleCandidate};
use crate::{Bundle, BundleId};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

const STORE_SCHEMA_VERSION: u8 = 1;
const DEFAULT_MAX_BUNDLES: usize = 4096;
const BASE_RETRY_MS: u64 = 2_000;
const MAX_RETRY_MS: u64 = 300_000;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredBundle {
    pub bundle: Bundle,
    pub envelope: Vec<u8>,
    pub stored_at_ms: u64,
    pub attempts: u32,
    pub next_attempt_at_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct StoreState {
    schema_version: u8,
    bundles: BTreeMap<BundleId, StoredBundle>,
}

impl Default for StoreState {
    fn default() -> Self {
        Self {
            schema_version: STORE_SCHEMA_VERSION,
            bundles: BTreeMap::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EnqueueOutcome {
    Stored,
    Duplicate,
    Expired,
    Full,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AttemptOutcome {
    Delivered,
    Deferred { next_attempt_at_ms: u64 },
    Missing,
}

#[derive(Debug)]
pub struct BundleStore {
    path: PathBuf,
    max_bundles: usize,
    state: StoreState,
    scheduler: BundleScheduler,
}

impl BundleStore {
    pub fn load(path: impl Into<PathBuf>) -> Result<Self> {
        Self::load_with_limit(path, DEFAULT_MAX_BUNDLES)
    }

    pub fn load_with_limit(path: impl Into<PathBuf>, max_bundles: usize) -> Result<Self> {
        let path = path.into();
        let state = if path.exists() {
            let raw = fs::read_to_string(&path)
                .with_context(|| format!("read bundle store {}", path.display()))?;
            let state: StoreState = serde_json::from_str(&raw)
                .with_context(|| format!("parse bundle store {}", path.display()))?;
            if state.schema_version != STORE_SCHEMA_VERSION {
                anyhow::bail!(
                    "unsupported bundle store schema {} (expected {})",
                    state.schema_version,
                    STORE_SCHEMA_VERSION
                );
            }
            state
        } else {
            StoreState::default()
        };

        Ok(Self {
            path,
            max_bundles: max_bundles.max(1),
            state,
            scheduler: BundleScheduler,
        })
    }

    pub fn len(&self) -> usize {
        self.state.bundles.len()
    }

    pub fn is_empty(&self) -> bool {
        self.state.bundles.is_empty()
    }

    pub fn max_bundles(&self) -> usize {
        self.max_bundles
    }

    pub fn contains(&self, id: &str) -> bool {
        self.state.bundles.contains_key(id)
    }

    pub fn get(&self, id: &str) -> Option<&StoredBundle> {
        self.state.bundles.get(id)
    }

    pub fn enqueue(
        &mut self,
        bundle: Bundle,
        envelope: Vec<u8>,
        now_ms: u64,
    ) -> EnqueueOutcome {
        self.prune_expired(now_ms);

        if bundle.is_expired_at(now_ms) {
            return EnqueueOutcome::Expired;
        }
        if self.state.bundles.contains_key(&bundle.id) {
            return EnqueueOutcome::Duplicate;
        }
        if self.state.bundles.len() >= self.max_bundles {
            return EnqueueOutcome::Full;
        }

        let id = bundle.id.clone();
        self.state.bundles.insert(
            id,
            StoredBundle {
                bundle,
                envelope,
                stored_at_ms: now_ms,
                attempts: 0,
                next_attempt_at_ms: now_ms,
            },
        );
        EnqueueOutcome::Stored
    }

    pub fn ready(&self, now_ms: u64, limit: usize) -> Vec<&StoredBundle> {
        let mut candidates: Vec<ScheduleCandidate> = self
            .state
            .bundles
            .values()
            .filter(|item| {
                !item.bundle.is_expired_at(now_ms)
                    && item.bundle.can_forward()
                    && item.next_attempt_at_ms <= now_ms
            })
            .map(|item| ScheduleCandidate {
                id: item.bundle.id.clone(),
                priority: item.bundle.priority,
                created_at_ms: item.bundle.created_at_ms,
                attempts: item.attempts,
            })
            .collect();

        self.scheduler.order(&mut candidates);
        candidates
            .into_iter()
            .take(limit)
            .filter_map(|candidate| self.state.bundles.get(&candidate.id))
            .collect()
    }

    pub fn record_attempt(
        &mut self,
        id: &str,
        now_ms: u64,
        delivered: bool,
    ) -> AttemptOutcome {
        if delivered {
            return if self.state.bundles.remove(id).is_some() {
                AttemptOutcome::Delivered
            } else {
                AttemptOutcome::Missing
            };
        }

        let Some(item) = self.state.bundles.get_mut(id) else {
            return AttemptOutcome::Missing;
        };

        item.attempts = item.attempts.saturating_add(1);
        let delay = retry_delay_ms(item.attempts);
        item.next_attempt_at_ms = now_ms.saturating_add(delay);
        AttemptOutcome::Deferred {
            next_attempt_at_ms: item.next_attempt_at_ms,
        }
    }

    pub fn prune_expired(&mut self, now_ms: u64) -> usize {
        let before = self.state.bundles.len();
        self.state
            .bundles
            .retain(|_, item| !item.bundle.is_expired_at(now_ms));
        before - self.state.bundles.len()
    }

    pub fn save(&self) -> Result<()> {
        if let Some(parent) = self.path.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent)
                    .with_context(|| format!("create bundle store directory {}", parent.display()))?;
            }
        }

        let tmp = temporary_path(&self.path);
        let bytes = serde_json::to_vec_pretty(&self.state)?;
        fs::write(&tmp, bytes)
            .with_context(|| format!("write temporary bundle store {}", tmp.display()))?;

        if self.path.exists() {
            fs::remove_file(&self.path)
                .with_context(|| format!("replace bundle store {}", self.path.display()))?;
        }
        fs::rename(&tmp, &self.path)
            .with_context(|| format!("commit bundle store {}", self.path.display()))?;
        Ok(())
    }
}

fn retry_delay_ms(attempts: u32) -> u64 {
    let shift = attempts.saturating_sub(1).min(8);
    BASE_RETRY_MS
        .saturating_mul(1u64 << shift)
        .min(MAX_RETRY_MS)
}

fn temporary_path(path: &Path) -> PathBuf {
    let mut tmp = path.as_os_str().to_os_string();
    tmp.push(".tmp");
    PathBuf::from(tmp)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Priority;
    use std::time::Duration;

    fn bundle(priority: Priority) -> Bundle {
        Bundle::new(
            b"opaque-test-envelope",
            Duration::from_secs(60),
            4,
            priority,
        )
    }

    #[test]
    fn queue_orders_priority_and_removes_delivered_bundle() {
        let path = test_path("queue-order");
        cleanup(&path);
        let mut store = BundleStore::load_with_limit(&path, 8).unwrap();
        let routine = bundle(Priority::Routine);
        let urgent = bundle(Priority::Urgent);
        let now = routine.created_at_ms.max(urgent.created_at_ms);

        assert_eq!(store.enqueue(routine, vec![1], now), EnqueueOutcome::Stored);
        assert_eq!(
            store.enqueue(urgent.clone(), vec![2], now),
            EnqueueOutcome::Stored
        );

        let ready = store.ready(now, 8);
        assert_eq!(ready[0].bundle.id, urgent.id);
        drop(ready);

        assert_eq!(
            store.record_attempt(&urgent.id, now, true),
            AttemptOutcome::Delivered
        );
        assert!(!store.contains(&urgent.id));
        cleanup(&path);
    }

    #[test]
    fn failed_delivery_uses_bounded_backoff() {
        let path = test_path("backoff");
        cleanup(&path);
        let mut store = BundleStore::load_with_limit(&path, 8).unwrap();
        let item = bundle(Priority::Important);
        let now = item.created_at_ms;
        let id = item.id.clone();
        store.enqueue(item, vec![9], now);

        let first = store.record_attempt(&id, now, false);
        assert_eq!(
            first,
            AttemptOutcome::Deferred {
                next_attempt_at_ms: now + BASE_RETRY_MS
            }
        );
        assert!(store.ready(now + 1, 1).is_empty());
        assert_eq!(store.ready(now + BASE_RETRY_MS, 1).len(), 1);
        cleanup(&path);
    }

    #[test]
    fn persistence_round_trip_preserves_envelope_and_attempt_state() {
        let path = test_path("roundtrip");
        cleanup(&path);
        let id;
        let now;
        {
            let mut store = BundleStore::load_with_limit(&path, 8).unwrap();
            let item = bundle(Priority::Important);
            now = item.created_at_ms;
            id = item.id.clone();
            store.enqueue(item, vec![4, 5, 6], now);
            store.record_attempt(&id, now, false);
            store.save().unwrap();
        }

        let store = BundleStore::load_with_limit(&path, 8).unwrap();
        let restored = store.get(&id).unwrap();
        assert_eq!(restored.envelope, vec![4, 5, 6]);
        assert_eq!(restored.attempts, 1);
        cleanup(&path);
    }

    #[test]
    fn capacity_is_bounded() {
        let path = test_path("capacity");
        cleanup(&path);
        let mut store = BundleStore::load_with_limit(&path, 1).unwrap();
        let a = bundle(Priority::Routine);
        let now = a.created_at_ms;
        let b = bundle(Priority::Urgent);

        assert_eq!(store.enqueue(a, vec![1], now), EnqueueOutcome::Stored);
        assert_eq!(store.enqueue(b, vec![2], now), EnqueueOutcome::Full);
        assert_eq!(store.len(), 1);
        cleanup(&path);
    }

    fn test_path(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!("sombra-{name}-{}.json", std::process::id()))
    }

    fn cleanup(path: &Path) {
        let _ = fs::remove_file(path);
        let _ = fs::remove_file(temporary_path(path));
    }
}
