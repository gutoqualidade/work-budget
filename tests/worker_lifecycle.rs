// SPDX-License-Identifier: GPL-3.0-only
use std::collections::BTreeSet;
use std::sync::mpsc::{self, Receiver, Sender, TryRecvError};
use std::thread::{self, JoinHandle};
use std::time::Duration;
use work_budget::{Cache, Jobs};

const DEADLINE: Duration = Duration::from_secs(5);

struct Worker {
    result: Receiver<&'static str>,
    handle: Option<JoinHandle<()>>,
}

struct Control {
    start: Sender<()>,
    finished: Receiver<()>,
}

fn spawn(value: &'static str) -> (Worker, Control) {
    let (start, go) = mpsc::channel();
    let (send_result, result) = mpsc::channel();
    let (done, finished) = mpsc::channel();
    let handle = thread::spawn(move || {
        go.recv_timeout(DEADLINE).expect("worker was not released");
        send_result.send(value).unwrap();
        done.send(()).unwrap();
    });
    (
        Worker {
            result,
            handle: Some(handle),
        },
        Control { start, finished },
    )
}

impl Control {
    fn complete(self) {
        self.start.send(()).unwrap();
        self.finished
            .recv_timeout(DEADLINE)
            .expect("worker did not complete");
    }
}

fn poll(worker: &mut Worker) -> Option<&'static str> {
    match worker.result.try_recv() {
        Ok(value) => {
            // The slot represents a thread: join it before reporting completion.
            worker.handle.take().unwrap().join().unwrap();
            Some(value)
        }
        Err(TryRecvError::Empty) => None,
        Err(TryRecvError::Disconnected) => panic!("worker exited without a result"),
    }
}

#[test]
fn late_obsolete_thread_cannot_overwrite_fresh_cached_result() {
    let mut jobs = Jobs::new(2);
    let mut cache = Cache::new(1);
    let mut old_control = None;
    assert!(jobs.submit("preview", || {
        let (worker, control) = spawn("obsolete");
        old_control = Some(control);
        worker
    }));
    jobs.invalidate();
    let mut new_control = None;
    assert!(jobs.submit("preview", || {
        let (worker, control) = spawn("fresh");
        new_control = Some(control);
        worker
    }));
    assert_eq!(jobs.available(), 0);
    assert!(!jobs.submit("other", || panic!("full tracker launched a worker")));
    assert!(jobs.poll(poll).is_empty());

    new_control.unwrap().complete();
    let completed = jobs.poll(poll);
    assert_eq!(completed, vec![("preview", "fresh")]);
    for (key, value) in completed {
        cache.insert(key, value);
    }
    assert_eq!(jobs.available(), 1);
    assert_eq!(jobs.len(), 1);
    assert!(!jobs.contains(&"preview"));

    old_control.unwrap().complete();
    assert!(jobs.poll(poll).is_empty());
    assert_eq!(jobs.available(), 2);
    assert_eq!(cache.get(&"preview"), Some(&"fresh"));
}

#[test]
fn zero_capacity_cache_reports_pinned_excess_until_unpinned() {
    let mut cache = Cache::new(0);
    cache.insert("keep", 7);
    assert_eq!(cache.trim(&BTreeSet::from(["keep"])), 1);
    assert_eq!(cache.get(&"keep"), Some(&7));
    assert_eq!(cache.trim(&BTreeSet::new()), 0);
    assert!(cache.is_empty());
}
