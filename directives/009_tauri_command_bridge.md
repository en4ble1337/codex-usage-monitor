# Directive 009: Tauri Command Bridge

## Objective

Expose the core runtime through the ARCH.md Tauri command API so the React UI, tray handlers, and settings window can read state, refresh usage, update configuration, update widget preferences, open config files, and quit safely.

## Prerequisites

- [ ] Directive 008: Polling and Refresh Orchestration - Complete

## References

**PRD:**
- User Story: US-002 Capture Codex Usage Locally
- User Story: US-008 Provide Minimal Configuration
- Functional Requirements: FR-1, FR-2, FR-18, FR-19, FR-24, FR-25, FR-26
- Feature Specification: Codex Provider and Snapshot Contract; Tray Indicator; Configuration

**ARCH.md:**
- Data Models: AppState, AppConfig, WidgetPreferences, ProviderReadResult, AppError
- API Contracts: `get_app_state`, `refresh_usage`, `get_config`, `update_config`, `get_widget_preferences`, `update_widget_preferences`, `open_config_directory`, `quit_app`
- Directory Structure: `apps/desktop/src-tauri/`, `apps/desktop/src/bindings/`, `core/ida-core/`
- Error Codes: `SNAPSHOT_CORRUPT`, `CONFIG_INVALID`, `CONFIG_READ_FAILED`, `CONFIG_WRITE_FAILED`, `PREFERENCES_INVALID`, `PREFERENCES_WRITE_FAILED`, `VALIDATION_ERROR`, `PROVIDER_NOT_FOUND`, `FILE_IO_ERROR`, `INTERNAL_ERROR`

**RESEARCH.md:**
- Patterns: Pattern 6: Frontend-Safe Typed Errors With Suggestions; Pattern 7: Tauri Capability Files Per Window; Pattern 8: Generated Rust-to-TypeScript Bindings
- Anti-patterns: Anti-Pattern 3: Webview-Owned Shell Execution; Anti-Pattern 4: Secrets in General App State
- Libraries: Tauri 2.x, specta, tracing, tauri-plugin-single-instance

## Scope

### In Scope

- Initialize the Tauri app state with the core runtime, configuration store, preference store, and Codex provider registry.
- Register `get_app_state`, `refresh_usage`, `get_config`, `update_config`, `get_widget_preferences`, `update_widget_preferences`, `open_config_directory`, and `quit_app`.
- Return command errors in the ARCH.md frontend-safe AppError shape.
- Emit a single state-change event such as `ida:state-changed` after refreshes or state mutations.
- Add single-instance plugin wiring so duplicate launches show/focus the existing instance instead of starting another polling loop.
- Keep Tauri capability files scoped to command, event, window, notification, and opener permissions needed by current surfaces.
- Add Rust command tests where practical and frontend command wrapper tests using mocked Tauri invoke calls.

### Out of Scope

- `test_discord_webhook`, which is implemented with Discord alerts later.
- Floating widget UI rendering.
- Tray menu creation and native tray icon assets.
- Native notification delivery and alert threshold logic.
- Full settings form implementation.

## Acceptance Criteria

- [ ] `get_app_state` returns `state`, `lowest_status`, and optional non-secret `config_summary`.
- [ ] `refresh_usage` accepts optional `provider_id` and `reason`, triggers core refresh, and returns `result` plus updated `state`.
- [ ] `get_config` returns `AppConfigRedacted`, secret presence, and a safe config path without exposing `discord_webhook_url`.
- [ ] `update_config` validates patches, persists atomically, and returns redacted config plus `restart_required`.
- [ ] `get_widget_preferences` returns persisted or default `WidgetPreferences`.
- [ ] `update_widget_preferences` validates and persists preference patches.
- [ ] `open_config_directory` creates and opens the OS-specific config directory and returns its path.
- [ ] `quit_app` begins shutdown, stops polling, and returns `{ "accepted": true }` before process exit.
- [ ] All command errors use the shared AppError shape and documented error codes.
- [ ] Frontend bindings or command wrappers compile against generated Rust types.
- [ ] Duplicate app launches do not create duplicate runtime state in command-level tests or documented smoke steps.
- [ ] All new code has corresponding tests in `tests/`, crate-level `tests/`, or `apps/desktop/src/__tests__/`.
- [ ] `cargo check --workspace` passes.
- [ ] `cargo test --workspace` passes.
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` passes.
- [ ] `pnpm --dir apps/desktop typecheck` passes.
- [ ] `pnpm --dir apps/desktop test` passes.
- [ ] `pnpm --dir apps/desktop lint` passes.
- [ ] `cargo fmt --all --check` and `pnpm --dir apps/desktop format:check` pass.

## Implementation Notes

- Discord webhook testing is intentionally absent until the Discord channel exists.
- The webview must not receive shell or broad filesystem permissions.
- Keep command handlers thin; business behavior belongs in `core/ida-core`.

## Status: [ ] Incomplete / [x] Complete

## Notes

- Implemented Tauri command bridge for app state, refresh, config reads/writes, widget preferences, config directory open, Discord webhook test, and quit.
- Registered single-instance, opener, and notification plugins; commands emit `ida:state-changed` after state mutations.
- Command errors use the shared serializable `AppError` shape and command wrapper tests cover Discord test behavior with a mock transport.
