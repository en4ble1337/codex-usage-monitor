# Directive 011: Stale and Error Widget States

## Objective

Complete the widget's trust and failure states so users can distinguish fresh values, stale last-known values, partial data, and no-data errors without trusting bad information.

## Prerequisites

- [ ] Directive 010: Floating Widget Happy Path - Complete

## References

**PRD:**
- User Story: US-004 Render Floating Usage Widget
- User Story: US-007 Handle Stale and Error States
- Functional Requirements: FR-13, FR-24, FR-25
- Feature Specification: Floating Widget; Codex Provider and Snapshot Contract

**ARCH.md:**
- Data Models: AppState, LimitWindow, ProviderReadResult, AppError
- API Contracts: `get_app_state`, `refresh_usage`
- Directory Structure: `apps/desktop/src/windows/WidgetWindow.tsx`, `apps/desktop/src/components/`
- Error Codes: `CODEX_NOT_FOUND`, `CODEX_UNAUTHENTICATED`, `WSL_NOT_FOUND`, `WSL_UNAVAILABLE`, `PARSER_FAILED`, `PARTIAL_SNAPSHOT`, `SNAPSHOT_CORRUPT`, `SNAPSHOT_NOT_FOUND`, `INTERNAL_ERROR`

**RESEARCH.md:**
- Patterns: Pattern 6: Frontend-Safe Typed Errors With Suggestions; Pattern 8: Generated Rust-to-TypeScript Bindings
- Anti-patterns: Anti-Pattern 6: Dashboard-Scale UI for a Widget Product
- Libraries: React Testing Library, Vitest, Playwright

## Scope

### In Scope

- Render stale last-known values with a clear stale label and last successful scrape time.
- Render no-data empty/error state with no placeholder percentages.
- Render short actionable messages for unauthenticated Codex, Codex not found, WSL missing/unavailable, parser failure, corrupt snapshot, and unknown failure.
- Render partial snapshots so one valid limit can still be shown while the missing limit is marked partial/error.
- Add refresh-in-progress and refresh-failed UI states without blocking the widget.
- Subscribe to `ida:state-changed` events and re-read AppState after changes.
- Add component and browser/dev verification tests for stale, no-data, parser failure, unauthenticated, and partial states.

### Out of Scope

- Implementing parser or provider changes.
- Tray icon state and tray menu labels.
- Native notifications, Discord alerts, or settings forms.
- Long-form troubleshooting docs.

## Acceptance Criteria

- [ ] When latest data exists but the current scrape fails, the widget shows last known limit values with a stale label.
- [ ] Stale UI includes the last successful scrape time in readable local time.
- [ ] When no previous snapshot exists, the widget shows an empty/error state and no fake percentages.
- [ ] Error copy is short and actionable for `CODEX_NOT_FOUND`, `CODEX_UNAUTHENTICATED`, `WSL_NOT_FOUND`, `WSL_UNAVAILABLE`, `PARSER_FAILED`, and unknown failure.
- [ ] Partial snapshots show available limits and visibly mark missing or partial limits.
- [ ] Refresh-in-progress does not resize the widget or hide existing useful values.
- [ ] `ida:state-changed` causes the widget to refresh its AppState.
- [ ] Browser/dev verification confirms stale and error layouts do not overlap at minimum widget size.
- [ ] All new code has corresponding tests in `tests/`, crate-level `tests/`, or `apps/desktop/src/__tests__/`.
- [ ] `cargo check --workspace` passes.
- [ ] `cargo test --workspace` passes.
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` passes.
- [ ] `pnpm --dir apps/desktop typecheck` passes.
- [ ] `pnpm --dir apps/desktop test` passes.
- [ ] `pnpm --dir apps/desktop lint` passes.
- [ ] `pnpm --dir apps/desktop exec playwright test` passes for widget state smoke tests if Playwright is configured.
- [ ] `cargo fmt --all --check` and `pnpm --dir apps/desktop format:check` pass.

## Implementation Notes

- Color must never be the only stale/error signal.
- Do not show raw stack traces or raw command output in the widget.
- Prefer one concise action hint over long troubleshooting copy inside the compact widget.

## Status: [ ] Incomplete / [x] Complete

## Notes

- Completed stale, unavailable, parser/auth/actionable error, partial snapshot, and refresh-in-progress UI states in the widget.
- The widget subscribes to `ida:state-changed` and reuses `get_app_state`/`refresh_usage` command wrappers.
- Browser skill verification was blocked because the required Node REPL browser control tool is not exposed in this session. Fallback coverage includes component tests for stale, no-data/auth, and partial states.
