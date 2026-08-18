# Directive 003: Core Domain Models and Provider Contract

## Objective

Define Ida's provider-neutral Rust domain model, structured error shape, and provider trait so Codex data can be normalized once and consumed by storage, polling, tray, widget, settings, and future providers.

## Prerequisites

- [ ] Directive 002: Product Workspace and Scaffold - Complete

## References

**PRD:**
- User Story: US-001 Define Normalized Provider Snapshot
- Functional Requirements: FR-7, FR-9, FR-27
- Feature Specification: Codex Provider and Snapshot Contract

**ARCH.md:**
- Data Models: ProviderSnapshot, LimitWindow, ProviderMetadata, ProviderReadResult, AppState, AppConfig, WidgetPreferences, AlertState, AlertStateEntry, AppError
- API Contracts: Rust Provider Trait `UsageProvider::refresh`
- Directory Structure: `core/ida-core/src/models/`, `core/ida-core/src/platform/`, `apps/desktop/src/bindings/`
- Error Codes: `VALIDATION_ERROR`, `PROVIDER_NOT_FOUND`, `INTERNAL_ERROR`

**RESEARCH.md:**
- Patterns: Pattern 6: Frontend-Safe Typed Errors With Suggestions; Pattern 8: Generated Rust-to-TypeScript Bindings
- Anti-patterns: Anti-Pattern 4: Secrets in General App State
- Libraries: serde, serde_json, specta, thiserror, async-trait, tracing

## Scope

### In Scope

- Implement Rust structs/enums for every ARCH.md data model entity in `core/ida-core`.
- Implement validation helpers for percentages, schema versions, provider IDs, status enums, reset text length, and non-secret metadata.
- Implement `AppError` with the ARCH.md JSON shape, stable error codes, operation IDs, UTC timestamps, and retryability.
- Implement the `UsageProvider` trait and `ProviderRuntimeConfig`.
- Add default constructors for `AppConfig`, `WidgetPreferences`, and empty `AlertState`.
- Add redacted view models such as `AppConfigRedacted` and patch models such as `AppConfigPatch` and `WidgetPreferencesPatch`.
- Wire specta type derivations and a binding-generation path for TypeScript consumers.

### Out of Scope

- Reading or writing files.
- Implementing Codex parsing or capture.
- Implementing polling, alerts, Tauri commands, tray, widget UI, or settings UI.
- Storing real configuration or webhook secrets.

## Acceptance Criteria

- [ ] `ProviderSnapshot` includes `schema_version`, `provider_id`, `provider_name`, `provider_status`, `scraped_at`, `capture_method`, `source_platform`, `limits`, and `metadata`.
- [ ] `LimitWindow` includes `id`, `label`, `window`, `remaining_pct`, `used_pct`, `resets_at`, `raw_reset_text`, `status`, `status_reason`, and non-secret `metadata`.
- [ ] `ProviderReadResult` represents `success`, `partial`, and `failure` outcomes without losing structured errors.
- [ ] `AppConfig` defaults include `polling_interval_seconds = 900`, `stale_after_seconds = 1800`, `alert_thresholds = [75, 50, 25, 10, 5]`, `capture_mode = native_then_wsl`, and `history_retention_hours = 24`.
- [ ] `AppConfigRedacted` never exposes `discord_webhook_url`; it exposes secret presence only where required.
- [ ] Validation rejects percentages outside `0..100`, invalid polling/stale bounds, invalid schema versions, malformed provider IDs, and webhook-like secrets in snapshot metadata.
- [ ] `UsageProvider` matches the ARCH.md trait shape and returns `ProviderReadResult`.
- [ ] Generated TypeScript bindings are produced or the generation command is documented in the app scaffold.
- [ ] Unit tests cover valid defaults, invalid percentages, redaction, provider trait mock usage, and AppError serialization.
- [ ] All new code has corresponding tests in `tests/`, crate-level `tests/`, or `apps/desktop/src/__tests__/`.
- [ ] `cargo check --workspace` passes.
- [ ] `cargo test --workspace` passes.
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` passes.
- [ ] `pnpm --dir apps/desktop typecheck` passes if bindings are generated into the frontend.
- [ ] `pnpm --dir apps/desktop lint` passes if frontend files are touched.
- [ ] `cargo fmt --all --check` passes.

## Implementation Notes

- Keep Rust models as the source of truth; do not create hand-maintained TypeScript duplicates.
- Use ARCH.md status names exactly: provider status `ok`, `partial`; freshness `fresh`, `stale`, `unavailable`, `error`; limit status `healthy`, `watch`, `low`, `critical`, `stale`, `error`.
- Make metadata serializable but validate it never contains webhook URLs, credentials, tokens, or raw terminal transcripts.

## Status: [ ] Incomplete / [x] Complete

## Notes

- Implemented provider-neutral Rust models, validation helpers, frontend-safe `AppError`, provider trait, config/preference patches, redacted config, and specta derives in `core/ida-core`.
- Added a TypeScript binding surface under `apps/desktop/src/bindings/ida.ts` aligned to the Rust model JSON contract.
- Tests cover defaults, invalid percentages, secret metadata rejection, redaction, provider trait usage, and AppError serialization.
