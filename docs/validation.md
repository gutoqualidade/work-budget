# Validation record

Date: 2026-10-15. Platform: Linux x86_64. Toolchain: Rust/Cargo 1.97.1,
Clippy 0.1.97. No older compiler or other platform was tested.

`bash scripts/check.sh` passed in the working directory and in a fresh temporary
copy containing only the prepared project files, with no build artifacts or
neighboring workspace. The temporary copy used the same installed toolchain;
this is an isolated-checkout check, not an independent machine or GitHub runner.

| Check | Observed result |
|---|---|
| `cargo fmt --all -- --check` | Passed |
| `cargo clippy --all-targets --offline --locked -- -D warnings` | Passed, no warnings |
| `cargo test --all-targets --offline --locked` | 3 unit + 2 integration tests passed |
| `cargo test --doc --offline --locked` | 1 compiled README example passed |
| `RUSTDOCFLAGS='-D warnings' cargo doc --no-deps --offline --locked` | Passed |
| `cargo run --quiet --offline --locked --example lifecycle` | Output exactly matched `docs/demo-output.txt` |
| Cargo metadata | One package, zero external crate dependencies, `publish = false` |
| Original source hash | Unchanged after extraction |

Total: **6 tests passed, zero failed or ignored**. The demo comparison is an
additional executable check, not an extra Rust test. Raw command output and the
isolated input manifest are retained in the local preparation review directory.

## CI and maintenance status

`.github/workflows/ci.yml` calls the same script with the same toolchain on an
Ubuntu runner. Checkout is pinned to a commit and its token has read-only content
permission. An Actions dependency-update configuration is prepared.

At the time of the local validation record, remote CI and Dependabot had not run.
The new repository will have no established maintenance history. Its initial
commit is a baseline, not evidence of sustained maintenance. No release, benchmark, security
audit, production deployment or long-running stability claim is made.
