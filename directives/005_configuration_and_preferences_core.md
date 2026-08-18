# Directive 005: Configuration and Widget Preferences Core

## Objective

Implement validated local configuration and widget preference persistence so polling, stale thresholds, alerts, Discord settings, and window behavior can be changed without editing source code.

## Prerequisites

- [ ] Directive 004: Local Snapshot Storage and App State - Complete

## References

**PRD:**
- User Story: US-008 Provide Minimal Configuration
- Functional Requirements: FR-9, FR-26
- Feature Specification: Configuration

**ARCH.md:**
- Data Models: AppConfig, WidgetPreferences, AppError
- API Contracts: None
- Directory Structure: `core/ida-core/src/models/`, `core/ida-core/src/storage/`
- Error Codes: `VALIDATION_ERROR`, `CONFIG_INVALID`, `CONFIG_READ_FAILED`, `CONFIG_WRITE_FAILED`, `PREFERENCES_INVALID`, `PREFERENCES_WRITE_FAILED`, `DISCORD_WEBHOOK_INVALID`, `FILE_IO_ERROR`

**RESEARCH.md:**
- Patterns: Pattern 3: Atomic Local JSON Writes; Pattern 6: Frontend-Safe Typed Errors With Suggestions
- Anti-patterns: Anti-Pattern 2: Direct File Writes for Critical State; Anti-Pattern 4: Secrets in General App State
- Libraries: serde_json, thiserror, tempfile

## Scope

### In Scope

- Implement `AppConfig` file read, default creation, validation, patch application, and atomic write behavior.
- Implement `WidgetPreferences` file read, default creation, validation, patch application, and atomic write behavior.
- Implement redacted config reads with secret presence for `discord_webhook_url`.
- Validate Discord webhook URLs as HTTPS Discord webhook endpoints when saving or testing values.
- Support clearing a stored Discord webhook via explicit secret update.
- Add tests for defaults, patch validation, redaction, malformed config recovery, malformed preferences recovery, and atomic writes.

### Out of Scope

- Building the settings UI.
- Implementing Tauri commands for config or preferences.
- Sending Discord webhooks.
- Applying preferences to a real Tauri window.
- Reading or modifying the prototype `local/.env`.

## Acceptance Criteria

- [ ] `AppConfig` persists outside the repo at the ARCH.md config path, with test overrides available.
- [ ] `polling_interval_seconds` validates `60..86400`.
- [ ] `stale_after_seconds` validates `120..172800`.
- [ ] `alert_thresholds` validate as unique descending integers in `0..100`.
- [ ] `discord_webhook_url` is stored only in local config and never returned by redacted config reads.
- [ ] Invalid Discord webhook URLs return `DISCORD_WEBHOOK_INVALID`.
- [ ] `WidgetPreferences` validates width `280..800`, height `160..600`, visibility, always-on-top, and nullable position/display fields.
- [ ] Invalid config and preference files produce `CONFIG_INVALID` or `PREFERENCES_INVALID` without panics.
- [ ] Unit tests cover patch behavior and secret clearing.
- [ ] All new code has corresponding tests in `tests/`, crate-level `tests/`, or `apps/desktop/src/__tests__/`.
- [ ] `cargo check --workspace` passes.
- [ ] `cargo test --workspace` passes.
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` passes.
- [ ] `cargo fmt --all --check` passes.

## Implementation Notes

- Prefer a single reusable atomic JSON helper from Directive 004.
- Redacted config responses should show secret presence and safe config paths, not secret values.
- Leave room for an environment variable override such as `IDA_DISCORD_WEBHOOK_URL`, but UI-saved config remains the default user path.

## Status: [ ] Incomplete / [x] Complete

## Notes

- Implemented local config and widget preference stores with default creation, validation, patch application, atomic writes, and malformed-file errors.
- Discord webhook URL validation accepts HTTPS Discord webhook endpoints only; redacted config exposes presence/masked state without returning the secret.
- Tests cover config patching, malformed config/preference files, invalid webhook validation, and explicit webhook clearing.
