# work-budget

Track in-flight work and discard obsolete results without releasing capacity
while old work is still running. This Rust library provides bookkeeping for a
host-owned executor and a small cache; it does not start threads or schedule jobs.

**Status:** experimental release. GPL-3.0-only. No external crate dependencies. Tested with Rust 1.97.1 on Linux; no minimum
supported Rust version or cross-platform support claim has been established.

## Example

An invalidation makes results obsolete, but keeps their work slots occupied:

```rust
use work_budget::Jobs;

let mut jobs = Jobs::new(1);
assert!(jobs.submit("preview", || "old result"));
jobs.invalidate();
assert_eq!(jobs.available(), 0);
assert!(!jobs.submit("preview", || "would exceed capacity"));

// The host reports completion. The obsolete result is discarded.
assert!(jobs.poll(|result| Some(*result)).is_empty());
assert_eq!(jobs.available(), 1);

assert!(jobs.submit("preview", || "new result"));
assert_eq!(jobs.poll(|result| Some(*result)), vec![("preview", "new result")]);
```

## Run locally

Clone the repository with Git. With Rust 1.97.1, Cargo, rustfmt, Clippy and Bash available:

```sh
git clone https://github.com/gutoqualidade/work-budget.git
cd work-budget
cargo run --offline --locked --example lifecycle
bash scripts/check.sh
```

The demo prints the slot count, the discarded obsolete completion and the
published fresh value. Its complete expected output is in
[docs/demo-output.txt](docs/demo-output.txt). The check script also compares the
actual demo output with that file. It needs no network after toolchain setup.
The crate is not published on crates.io.

## Behavior

- `Jobs` rejects duplicate keys within a generation and submissions when full.
  Rejected submissions never invoke the spawn closure.
- Invalidation advances the generation. Old tasks still occupy slots until the
  host reports their completion. Polling returns only current-generation results.
- `Cache` ages entries on insertion and successful access. Explicit `trim`
  removes the least recently used unpinned entries and returns any remaining
  excess when pins prevent reaching the target capacity.

## Architecture and validation

The host owns execution, completion detection, errors and pinning policy.
`Jobs` and `Cache` own only their bookkeeping state. See the
[boundary diagram](docs/architecture.md), the
[decision to retain in-flight reservations](docs/decisions/0001-retain-reservations.md)
and the [validation record](docs/validation.md).

Tests cover obsolete results and errors, repeated invalidation, duplicate
submissions, cache eviction and pinning, and real thread completions arriving
in a different order from submission. The README example is a compiled doctest.
The CI workflow is prepared but has not run on GitHub.

## Limitations

- This is a count of tracked jobs, not a CPU, memory or rate limit. The host must
  submit every relevant worker through `Jobs` and poll regularly.
- Invalidation does not cancel, interrupt or join workers. A worker that never
  completes retains its slot. Dropping the tracker drops handles; the effect
  on running work depends on those handles. There are no deadlines or retries.
- The host must report a completion only when the resource represented by its
  slot can be released. Reporting a result early can undercount running work.
- Each tracker/cache requires a mutable owner. Cross-thread synchronization is
  the host's responsibility. No fairness guarantee or async runtime is provided.
- Spawn and poll callbacks must return normally. A panic after launching work
  can leave that work untracked; callback panic recovery is not supported.
- Cache insertion does not enforce capacity. Call `trim` explicitly. Pinned
  entries can keep the cache above its target, which the host must handle.
- Eviction scans entries for each removal; this is not a constant-time LRU.
  Generation/access counters panic on `u64` exhaustion; `Jobs::new(0)` panics.
- No throughput benchmark, long-running soak test or production certification
  has been performed. The API may change.

## Development and help

Run `bash scripts/check.sh` before proposing a change. Include a reproducer,
expected behavior and observed behavior for defects. See
[CONTRIBUTING.md](CONTRIBUTING.md). No public issue tracker or support commitment
exists while this remains a local draft.

## Licensing

Copyright (C) 2026 gutoqualidade. Licensed under GPL-3.0-only; see the `LICENSE` and `LICENSING.md` files for the
full terms and a plain-language summary of the scope. This license requires corresponding
source when distributing covered versions; private use can remain private.
