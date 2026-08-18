# Directive 007: Codex Capture Provider

## Objective

Implement the Codex provider capture path that attempts native Codex status capture first, falls back to WSL on Windows when configured, and returns normalized provider read results through the shared provider contract.

## Prerequisites

- [ ] Directive 005: Configuration and Widget Preferences Core - Complete
- [ ] Directive 006: Codex Parser and Prototype Compatibility - Complete

## References

**PRD:**
- User Story: US-002 Capture Codex Usage Locally
- Functional Requirements: FR-1, FR-2, FR-3, FR-4, FR-5, FR-6, FR-7, FR-27
- Feature Specification: Codex Provider and Snapshot Contract

**ARCH.md:**
- Data Models: ProviderSnapshot, ProviderReadResult, AppConfig, AppError
- API Contracts: Rust Provider Trait `UsageProvider::refresh`
- Directory Structure: `providers/codex/`, `core/ida-core/src/platform/`
- Error Codes: `CODEX_NOT_FOUND`, `CODEX_UNAUTHENTICATED`, `WSL_NOT_FOUND`, `WSL_UNAVAILABLE`, `CAPTURE_TIMEOUT`, `CAPTURE_FAILED`, `PARSER_FAILED`, `PARTIAL_SNAPSHOT`, `INTERNAL_ERROR`

**RESEARCH.md:**
- Patterns: Pattern 4: Async Polling Loop With Shutdown and Events; Pattern 6: Frontend-Safe Typed Errors With Suggestions
- Anti-patterns: Anti-Pattern 3: Webview-Owned Shell Execution
- Libraries: tokio, async-trait, tracing, thiserror

## Scope

### In Scope

- Implement process execution abstractions that run commands with argument arrays, timeouts, and test doubles.
- Implement native Codex capture according to the best available local command path from the prototype and architecture notes.
- Implement Windows WSL fallback when `capture_mode = native_then_wsl` or `wsl_only`.
- Detect and map missing native Codex, missing WSL, unavailable WSL, capture timeout, unauthenticated Codex, and parser failure.
- Populate `capture_method` as `native`, `wsl`, or `unknown`.
- Populate `source_platform` from the current platform.
- Implement `CodexProvider` as `UsageProvider`.
- Add tests for native success, native missing fallback to WSL, WSL missing, capture timeout, unauthenticated output, parser failure, and partial snapshot.

### Out of Scope

- Scheduling repeated polling.
- Persisting successful snapshots.
- Implementing Tauri commands or UI.
- Exposing shell execution to the React webview.
- Implementing Claude capture.

## Acceptance Criteria

- [ ] `CodexProvider::refresh` returns a successful `ProviderReadResult` for fixture-backed native capture.
- [ ] On Windows with `native_then_wsl`, native `CODEX_NOT_FOUND` attempts WSL before failing.
- [ ] `native_only` never attempts WSL, and `wsl_only` never attempts native Codex.
- [ ] Command execution uses executable plus argument arrays, not string-built shell commands.
- [ ] Capture has a bounded timeout and returns `CAPTURE_TIMEOUT` when exceeded.
- [ ] Missing WSL maps to `WSL_NOT_FOUND`; failed WSL execution maps to `WSL_UNAVAILABLE` or `CAPTURE_FAILED` as appropriate.
- [ ] Known sign-in failures map to `CODEX_UNAUTHENTICATED` with an actionable message.
- [ ] Successful provider snapshots include `capture_method`, `source_platform`, `provider_id = codex`, and `provider_name = Codex`.
- [ ] Unit tests use process test doubles and do not require real Codex, WSL, or network access.
- [ ] All new code has corresponding tests in `tests/`, crate-level `tests/`, or `apps/desktop/src/__tests__/`.
- [ ] `cargo check --workspace` passes.
- [ ] `cargo test --workspace` passes.
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` passes.
- [ ] `cargo fmt --all --check` passes.

## Implementation Notes

- Keep all capture code in Rust provider code. Do not add Tauri shell plugin permissions for Codex.
- Redact raw command output from user-facing errors and normal logs; include operation IDs for debugging.
- If the final Codex status command is uncertain, isolate it behind a small strategy object so fixture tests remain stable.

## Status: [ ] Incomplete / [x] Complete

## Notes

- Implemented Rust-side command execution abstraction with argument arrays, timeout handling, native Codex capture, and Windows WSL fallback.
- `CodexProvider` implements `UsageProvider` and maps native missing, WSL missing/unavailable, timeout, unauthenticated, parser failure, and partial snapshot cases into structured results.
- Tests use process doubles only; no real Codex, WSL, shell plugin, or network access is required.
