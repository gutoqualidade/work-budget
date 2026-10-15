//! Executor-independent bookkeeping for bounded derived-data workers.
//!
//! The host owns the executor and polls handles. Invalidating results never frees
//! a running worker's reservation: dropping a future need not preempt CPU work.
//! No ECS entities, meshes, world coordinates or scheduler are required here.
use std::collections::{BTreeMap, BTreeSet};

/// Logical cancellation with a physical outstanding-work limit. Keys identify
/// requests within one generation; each reset starts a new generation.
pub struct Jobs<K, T> {
    generation: u64,
    limit: usize,
    pending: BTreeMap<(u64, K), T>,
}
impl<K: Ord + Clone, T> Jobs<K, T> {
    pub fn new(limit: usize) -> Self {
        assert!(limit > 0, "worker limit must be positive");
        Self {
            generation: 0,
            limit,
            pending: BTreeMap::new(),
        }
    }
    pub fn len(&self) -> usize {
        self.pending.len()
    }
    pub fn is_empty(&self) -> bool {
        self.pending.is_empty()
    }
    pub fn available(&self) -> usize {
        self.limit - self.pending.len()
    }
    pub fn contains(&self, key: &K) -> bool {
        self.pending.contains_key(&(self.generation, key.clone()))
    }
    pub fn invalidate(&mut self) {
        self.generation = self
            .generation
            .checked_add(1)
            .expect("job generation exhausted");
    }
    /// Calls `spawn` only when capacity is available and the key is not pending
    /// in the current generation. A rejected request launches no work.
    /// The host must submit every worker through this tracker. `spawn` must
    /// return normally: work launched before a panic cannot be accounted for.
    pub fn submit(&mut self, key: K, spawn: impl FnOnce() -> T) -> bool {
        if self.available() == 0 || self.contains(&key) {
            return false;
        }
        self.pending.insert((self.generation, key), spawn());
        true
    }
    /// Complete jobs release reservations, including obsolete ones. Only current
    /// generation results are returned for publication/cache insertion. A job's
    /// error can be represented by `R = Result<_, _>` and follows the same rule.
    pub fn poll<R>(&mut self, mut poll: impl FnMut(&mut T) -> Option<R>) -> Vec<(K, R)> {
        let mut completed = Vec::new();
        for (key, task) in &mut self.pending {
            if let Some(result) = poll(task) {
                completed.push((key.clone(), result));
            }
        }
        let mut current = Vec::new();
        for ((generation, key), result) in completed {
            self.pending.remove(&(generation, key.clone()));
            if generation == self.generation {
                current.push((key, result));
            }
        }
        current
    }
}

/// Deterministic access-aged cache. Pinning is supplied by the host's residency
/// policy. If every entry is pinned, the host must reduce its working set first.
pub struct Cache<K, V> {
    entries: BTreeMap<K, (V, u64)>,
    clock: u64,
    limit: usize,
}
impl<K: Ord + Clone, V> Cache<K, V> {
    pub fn new(limit: usize) -> Self {
        Self {
            entries: BTreeMap::new(),
            clock: 0,
            limit,
        }
    }
    pub fn len(&self) -> usize {
        self.entries.len()
    }
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
    pub fn contains(&self, key: &K) -> bool {
        self.entries.contains_key(key)
    }
    pub fn clear(&mut self) {
        self.entries.clear();
        self.clock = 0;
    }
    fn next_age(&mut self) -> u64 {
        self.clock = self.clock.checked_add(1).expect("cache clock exhausted");
        self.clock
    }
    pub fn insert(&mut self, key: K, value: V) {
        let age = self.next_age();
        self.entries.insert(key, (value, age));
    }
    pub fn get(&mut self, key: &K) -> Option<&V> {
        let age = self.next_age();
        let (value, last) = self.entries.get_mut(key)?;
        *last = age;
        Some(value)
    }
    /// Returns the remaining excess; a nonzero value explicitly signals that the
    /// requested pinned working set exceeds this cache's capacity.
    pub fn trim(&mut self, pinned: &BTreeSet<K>) -> usize {
        while self.entries.len() > self.limit {
            let oldest = self
                .entries
                .iter()
                .filter(|(k, _)| !pinned.contains(k))
                .min_by_key(|(_, (_, age))| *age)
                .map(|(k, _)| k.clone());
            let Some(key) = oldest else {
                break;
            };
            self.entries.remove(&key);
        }
        self.entries.len().saturating_sub(self.limit)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    #[test]
    fn resets_retain_all_physical_reservations_until_completion() {
        let mut jobs = Jobs::new(4);
        let launches = Cell::new(0);
        for key in 0..4 {
            assert!(jobs.submit(key, || {
                launches.set(launches.get() + 1);
                false
            }));
        }
        for _ in 0..16 {
            jobs.invalidate();
            assert!(!jobs.submit(99, || {
                launches.set(999);
                true
            }));
            assert!(jobs.poll(|done| done.then_some(1)).is_empty());
            assert_eq!(jobs.len(), 4);
        }
        assert_eq!(launches.get(), 4);
        // The original CPU jobs finish. None of their stale results is publishable.
        assert!(jobs.poll(|_| Some(7)).is_empty());
        assert_eq!(jobs.available(), 4);
        assert!(jobs.submit(0, || true));
        assert_eq!(jobs.poll(|done| done.then_some(42)), vec![(0, 42)]);
    }

    #[test]
    fn duplicate_requests_and_obsolete_errors_do_not_leak_capacity_or_results() {
        let mut jobs = Jobs::new(2);
        assert!(jobs.submit(1, || 0));
        assert!(!jobs.submit(1, || panic!("duplicate spawned")));
        jobs.invalidate();
        assert!(jobs.submit(1, || 1));
        assert_eq!(
            jobs.poll(|v| Some(if *v == 0 { Err("obsolete") } else { Ok(5) })),
            vec![(1, Ok(5))]
        );
        assert!(jobs.is_empty());
        assert!(jobs.submit(2, || 2));
        assert_eq!(
            jobs.poll(|_| Some(Err::<(), _>("current failure"))),
            vec![(2, Err("current failure"))]
        );
        assert_eq!(jobs.available(), 2);
    }

    #[test]
    fn cache_preserves_pins_and_evicts_least_recent_access() {
        let mut cache = Cache::new(2);
        cache.insert(1, "a");
        cache.insert(2, "b");
        assert_eq!(cache.get(&1), Some(&"a"));
        cache.insert(3, "c");
        assert_eq!(cache.trim(&BTreeSet::new()), 0);
        assert!(!cache.contains(&2));
        cache.insert(4, "d");
        assert_eq!(cache.trim(&BTreeSet::from([1, 3, 4])), 1);
        assert_eq!(cache.trim(&BTreeSet::from([1])), 0);
        assert!(cache.contains(&1));
        cache.clear();
        assert!(cache.is_empty());
    }
}
