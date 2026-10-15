# 0001: Retain reservations until completion

Status: implemented in the local prototype.

## Problem

A host starts expensive derived work. Inputs change before a worker completes.
Its result is obsolete, but dropping the handle may not stop CPU work already
running. If invalidation immediately freed its slot, repeated invalidations
could start more workers than the intended outstanding-work limit.

## Decision

Assign submissions to a generation. Invalidation advances the generation without
removing handles. Poll all generations, release slots on reported completion and
return only results matching the current generation. Duplicate keys are rejected
within a generation, but can be submitted in a later generation if capacity exists.

## Alternatives and costs

- Clearing all handles is simpler and more responsive but cannot bound work that
  survives handle disposal.
- Requiring a preemptible executor would permit physical cancellation but adds
  an executor-specific contract that many CPU workloads cannot satisfy.
- Retaining reservations is conservative: obsolete work may block fresh work,
  and a stuck worker holds its slot indefinitely. Hosts needing deadlines,
  cooperative cancellation or process termination must implement those policies.

This tracker bounds the handles it admits, not arbitrary work launched outside
it. A host that reports completion before the represented resource is released,
or whose spawn callback starts work then panics, breaks the accounting contract.

## Verification

- Unit test `resets_retain_all_physical_reservations_until_completion` repeats
  invalidation while every slot is occupied and rejects extra launches.
- Unit test `duplicate_requests_and_obsolete_errors_do_not_leak_capacity_or_results`
  checks duplicate rejection and generation filtering for both values and errors.
- Integration test `late_obsolete_thread_cannot_overwrite_fresh_cached_result`
  starts actual threads, completes the new generation first, then completes the
  old one. The new value stays in the cache and slots recover after joins.

The thread test uses channel gates, not sleeps, to control completion order.
These tests demonstrate specified cases; they do not prove every possible
executor integration, absence of all concurrency bugs, or production readiness.
