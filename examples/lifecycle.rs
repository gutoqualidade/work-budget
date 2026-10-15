use std::collections::BTreeSet;
use work_budget::{Cache, Jobs};

fn main() {
    // Immediate values model completed handles; execution belongs to the host.
    let mut jobs = Jobs::new(1);
    let mut cache = Cache::new(1);
    assert!(jobs.submit("preview", || "old"));
    jobs.invalidate();
    println!(
        "After invalidation: {} tracked, {} available",
        jobs.len(),
        jobs.available()
    );
    let admitted = jobs.submit("preview", || "must not run");
    assert!(!admitted);
    println!("Replacement admitted while full: {admitted}");
    let stale = jobs.poll(|value| Some(*value));
    assert!(stale.is_empty());
    println!("Obsolete completions published: {}", stale.len());
    assert!(jobs.submit("preview", || "fresh"));
    for (key, value) in jobs.poll(|value| Some(*value)) {
        cache.insert(key, value);
    }
    println!("Cached preview: {}", cache.get(&"preview").unwrap());
    cache.insert("other", "retained");
    let excess = cache.trim(&BTreeSet::from(["preview", "other"]));
    assert_eq!(excess, 1);
    println!("Excess while both entries are pinned: {excess}");
    assert_eq!(cache.trim(&BTreeSet::from(["preview"])), 0);
    println!(
        "After unpinning: {} cached, {} available work slots",
        cache.len(),
        jobs.available()
    );
}
