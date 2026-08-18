# Directive 008: Polling and Refresh Orchestration

## Objective

Connect provider refresh, configuration, storage, stale detection, and AppState updates into a core service that supports startup refresh, scheduled polling, and non-overlapping manual refresh.

## Prerequisites

- [ ] Directive 004: Local Snapshot Storage and App State - Complete
- [ ] Directive 005: Configuration and Widget Preferences Core - Complete
- [ ] Directive 007: Codex Capture Provider - Complete

## References

**PRD:**
- User Story: US-002 Capture Codex Usage Locally
- User Story: US-003 Store Latest Snapshot Locally
- Functional Requirements: FR-1, FR-8, FR-24, FR-25
- Feature Specification: Codex Provider and Snapshot Contract; Configuration

**ARCH.md:**
- Data Models: ProviderReadResult, ProviderSnapshot, AppState, AppConfig, AppError
- API Contracts: `refresh_usage` service behavior before Tauri command wrapping
- Directory Structure: `core/ida-core/src/`, `core/ida-core/src/storage/`, `core/ida-core/src/platform/`
- Error Codes: `PROVIDER_NOT_FOUND`, `SNAPSHOT_CORRUPT`, `FILE_IO_ERROR`, `CAPTURE_TIMEOUT`, `PARSER_FAILED`, `INTERNAL_ERROR`

**RESEARCH.md:**
- Patterns: Pattern 4: Async Polling Loop With Shutdown and Events; Pattern 3: Atomic Local JSON Writes
- Anti-patterns: Anti-Pattern 1: Broad App Architecture by Copying a Large Product
- Libraries: tokio, tracing, async-trait

## Scope

### In Scope

- Implement a provider registry with Codex registered as the active MVP provider.
- Implement an `AppRuntime` or equivalent core orchestration service.
- Implement startup refresh, scheduled polling by `polling_interval_seconds`, and manual refresh reasons.
- Prevent overlapping provider captures when manual refresh and polling happen near the same time.
- Apply successful and partial provider results to latest snapshot, history, and AppState.
- Apply failed provider results to AppState without overwriting the latest successful snapshot.
- Emit or expose state-change hooks for the later Tauri layer.
- Add tests for startup refresh, manual refresh, scheduled refresh timing with paused time, failed scrape preserving latest, and provider-not-found behavior.

### Out of Scope

- Tauri command registration.
- Widget, tray, settings UI, native notifications, or Discord webhooks.
- Full desktop process lifecycle and quit behavior.
- Long-term analytics beyond the short history file.

## Acceptance Criteria

- [ ] Startup refresh can produce an AppState with fresh Codex limits when the provider succeeds.
- [ ] Manual refresh returns both `ProviderReadResult` and updated `AppState`.
- [ ] Scheduled polling respects `polling_interval_seconds` and updates `next_poll_at`.
- [ ] Concurrent manual and scheduled refresh calls do not run overlapping Codex captures.
- [ ] Failed refresh preserves latest successful snapshot and marks effective limits stale when applicable.
- [ ] No previous snapshot plus failed refresh returns an AppState with `freshness_status = error` or `unavailable` and a structured AppError.
- [ ] Unknown provider IDs return `PROVIDER_NOT_FOUND`.
- [ ] Storage failures from successful captures surface as structured errors and do not crash the runtime.
- [ ] Tests use mock providers and temp storage, not real Codex or WSL.
- [ ] All new code has corresponding tests in `tests/`, crate-level `tests/`, or `apps/desktop/src/__tests__/`.
- [ ] `cargo check --workspace` passes.
- [ ] `cargo test --workspace` passes.
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` passes.
- [ ] `cargo fmt --all --check` passes.

## Implementation Notes

- Use a shutdown channel or cancellation token so the Tauri app can stop polling cleanly later.
- Keep alert delivery hooks present but inactive; actual alert orchestration is implemented in a later directive.
- Emit only safe state summaries. Do not expose raw provider output.

## Status: [ ] Incomplete / [x] Complete

## Notes

- Implemented provider registry and `AppRuntime` orchestration for startup/manual/poll refresh reasons, non-overlapping refreshes, storage application, state updates, and next poll calculation.
- Refreshes preserve latest successful data on failure and surface unknown providers as `PROVIDER_NOT_FOUND`.
- Alert evaluation hooks are wired into successful/partial refresh paths.
