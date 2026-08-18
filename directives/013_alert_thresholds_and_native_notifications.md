# Directive 013: Alert Thresholds and Native Notifications

## Objective

Implement provider-neutral threshold crossing, alert deduplication, alert state persistence, and native Windows notification delivery so Ida warns users once per threshold and reset window.

## Prerequisites

- [ ] Directive 008: Polling and Refresh Orchestration - Complete

## References

**PRD:**
- User Story: US-006 Alert on Low Remaining Usage
- Functional Requirements: FR-20, FR-22, FR-23
- Feature Specification: Alerts

**ARCH.md:**
- Data Models: AppConfig, ProviderSnapshot, LimitWindow, AlertState, AlertStateEntry, AppError
- API Contracts: Alert orchestration invoked by refresh/polling services
- Directory Structure: `core/ida-core/src/alerts/`, `core/ida-core/src/platform/`
- Error Codes: `ALERT_STATE_WRITE_FAILED`, `NOTIFICATIONS_UNAVAILABLE`, `FILE_IO_ERROR`, `INTERNAL_ERROR`

**RESEARCH.md:**
- Patterns: Pattern 4: Async Polling Loop With Shutdown and Events; Pattern 6: Frontend-Safe Typed Errors With Suggestions
- Anti-patterns: Anti-Pattern 1: Broad App Architecture by Copying a Large Product
- Libraries: Tauri notification plugin 2.x, tracing, serde_json

## Scope

### In Scope

- Implement provider-neutral threshold crossing logic using `AppConfig.alert_thresholds`.
- Implement a generic alert channel trait or boundary with native notification as the first channel.
- Implement alert dedupe keyed by provider ID, limit ID, threshold, channel, and reset window key.
- Persist `AlertState` atomically and prune stale entries when reset windows change.
- Integrate alert evaluation into refresh/polling success and partial success paths.
- Implement native notification delivery behind a platform abstraction so non-Windows behavior can fail gracefully.
- Add tests for crossing thresholds downward, non-crossing updates, duplicate suppression, reset window changes, native channel failure, and alert state write failure.

### Out of Scope

- Discord webhook delivery.
- Settings UI for alert thresholds.
- Telegram implementation.
- Blocking widget updates when notifications fail.

## Acceptance Criteria

- [ ] Threshold crossing fires when remaining percentage moves from above a configured threshold to at or below it.
- [ ] Default thresholds are `[75, 50, 25, 10, 5]` when no user config exists.
- [ ] Native notification alerts include provider, limit window, remaining percentage, and reset time when available.
- [ ] The same provider/limit/threshold/channel/reset window does not alert repeatedly.
- [ ] Alert dedupe resets when `resets_at` or the raw reset text key changes.
- [ ] Failed native notification delivery records failure in alert state or logs but does not block AppState updates.
- [ ] Alert channel boundary is generic enough for Discord and future Telegram without changing threshold or dedupe logic.
- [ ] `AlertState` writes atomically and maps write failures to `ALERT_STATE_WRITE_FAILED`.
- [ ] Tests use mock alert channels and temp alert state storage.
- [ ] All new code has corresponding tests in `tests/`, crate-level `tests/`, or `apps/desktop/src/__tests__/`.
- [ ] `cargo check --workspace` passes.
- [ ] `cargo test --workspace` passes.
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` passes.
- [ ] `pnpm --dir apps/desktop typecheck` passes if Tauri notification plugin bindings are touched.
- [ ] `cargo fmt --all --check` passes.

## Implementation Notes

- Store only dedupe metadata in alert state. Do not store webhook URLs, provider credentials, or raw provider output.
- Native notification permission or availability problems should map to `NOTIFICATIONS_UNAVAILABLE` and remain non-fatal.
- Keep Telegram out of MVP; the boundary is the deliverable, not a Telegram channel.

## Status: [ ] Incomplete / [x] Complete

## Notes

- Implemented provider-neutral threshold crossing, per-channel/reset-window dedupe, atomic alert state persistence, pruning, and alert channel trait.
- Wired native notifications through a Tauri notification channel in the desktop runtime; tests use mock channels for deterministic delivery/dedupe assertions.
- Notification permission/availability failures map to `NOTIFICATIONS_UNAVAILABLE` and do not block AppState updates.
