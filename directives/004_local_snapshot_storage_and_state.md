# Directive 004: Local Snapshot Storage and App State

## Objective

Implement local latest snapshot and short history persistence, plus AppState assembly that preserves last known values during scrape failures and exposes corrupt or missing storage as structured state.

## Prerequisites

- [ ] Directive 003: Core Domain Models and Provider Contract - Complete

## References

**PRD:**
- User Story: US-003 Store Latest Snapshot Locally
- User Story: US-007 Handle Stale and Error States
- Functional Requirements: FR-8, FR-9, FR-24, FR-25
- Feature Specification: Codex Provider and Snapshot Contract

**ARCH.md:**
- Data Models: ProviderSnapshot, LimitWindow, ProviderReadResult, AppState, AppConfig, AppError
- API Contracts: None
- Directory Structure: `core/ida-core/src/storage/`, `tests/fixtures/`
- Error Codes: `SNAPSHOT_NOT_FOUND`, `SNAPSHOT_CORRUPT`, `HISTORY_WRITE_FAILED`, `FILE_IO_ERROR`, `INTERNAL_ERROR`

**RESEARCH.md:**
- Patterns: Pattern 3: Atomic Local JSON Writes; Pattern 6: Frontend-Safe Typed Errors With Suggestions
- Anti-patterns: Anti-Pattern 2: Direct File Writes for Critical State; Anti-Pattern 4: Secrets in General App State
- Libraries: serde_json, tempfile, tracing

## Scope

### In Scope

- Implement app directory resolution abstractions for config/state paths, with testable overrides for temp directories.
- Implement atomic JSON writes for `latest.json`.
- Implement append and retention trimming for `history.ndjson`.
- Implement read/validate behavior for latest snapshots, including missing and corrupt file handling.
- Implement `AppState` assembly from latest snapshot, current provider result, and `AppConfig`.
- Apply stale status to effective limits when latest data exceeds `stale_after_seconds` or the current scrape failed but a latest snapshot exists.
- Ensure failed scrapes never overwrite the latest successful snapshot.
- Add tests with sanitized fixture snapshots and corrupt storage files.

### Out of Scope

- Implementing AppConfig persistence.
- Implementing Codex parser, capture, polling loop, alerts, Tauri commands, or frontend rendering.
- Introducing SQLite or long-term analytics storage.
- Persisting webhook URLs or other secrets in snapshots or history.

## Acceptance Criteria

- [ ] A successful `ProviderReadResult` writes `latest.json` atomically and appends one line to `history.ndjson`.
- [ ] A failed `ProviderReadResult` updates AppState error context but does not replace `latest.json`.
- [ ] Missing latest snapshot returns `SNAPSHOT_NOT_FOUND` or an `AppState` with `freshness_status = unavailable` when appropriate.
- [ ] Corrupt latest snapshot files return `SNAPSHOT_CORRUPT` and do not crash.
- [ ] History trimming respects `history_retention_hours`.
- [ ] Effective limits are marked `stale` when the latest successful snapshot age exceeds `stale_after_seconds`.
- [ ] Storage tests prove snapshots and history do not include webhook URLs, OpenAI credentials, Anthropic credentials, API keys, or raw terminal transcripts.
- [ ] Atomic write tests prove temp files are cleaned up or safely ignored after successful writes.
- [ ] All new code has corresponding tests in `tests/`, crate-level `tests/`, or `apps/desktop/src/__tests__/`.
- [ ] `cargo check --workspace` passes.
- [ ] `cargo test --workspace` passes.
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` passes.
- [ ] `cargo fmt --all --check` passes.

## Implementation Notes

- Use temp-file-then-rename writes for JSON state.
- Keep filesystem paths outside the repository by default: `%LOCALAPPDATA%/Ida/state/latest.json` and `%LOCALAPPDATA%/Ida/state/history.ndjson` on Windows.
- Do not make stale detection depend on system local time formatting; compare UTC instants.

## Status: [ ] Incomplete / [x] Complete

## Notes

- Implemented OS app directory resolution, test overrides, atomic JSON writes, latest snapshot reads, history append/trimming, and AppState assembly.
- Failed provider results preserve the previous latest snapshot and mark effective limits stale when appropriate.
- Storage tests cover corrupt latest files, failed scrape preservation, stale detection, and secret-safe serialization.
