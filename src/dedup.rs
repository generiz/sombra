use crate::BundleId;
use std::collections::{HashSet, VecDeque};

#[derive(Debug, Clone)]
pub struct DedupCache {
    capacity: usize,
    order: VecDeque<BundleId>,
    seen: HashSet<BundleId>,
}

impl DedupCache {
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity: capacity.max(1),
            order: VecDeque::with_capacity(capacity.max(1)),
            seen: HashSet::with_capacity(capacity.max(1)),
        }
    }

    pub fn contains(&self, id: &str) -> bool {
        self.seen.contains(id)
    }

    pub fn remember(&mut self, id: BundleId) -> bool {
        if self.seen.contains(&id) {
            return false;
        }

        if self.order.len() == self.capacity {
            if let Some(oldest) = self.order.pop_front() {
                self.seen.remove(&oldest);
            }
        }

        self.seen.insert(id.clone());
        self.order.push_back(id);
        true
    }

    pub fn len(&self) -> usize {
        self.order.len()
    }

    pub fn is_empty(&self) -> bool {
        self.order.is_empty()
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn duplicate_is_rejected() {
        let mut cache = DedupCache::new(4);
        assert!(cache.remember("a".into()));
        assert!(!cache.remember("a".into()));
        assert_eq!(cache.len(), 1);
    }

    #[test]
    fn oldest_entry_is_evicted_when_full() {
        let mut cache = DedupCache::new(2);
        cache.remember("a".into());
        cache.remember("b".into());
        cache.remember("c".into());

        assert!(!cache.contains("a"));
        assert!(cache.contains("b"));
        assert!(cache.contains("c"));
    }
}
