# Directive 006: Codex Parser and Prototype Compatibility

## Objective

Port the useful Codex status parsing behavior from the prototype into the new Codex provider crate, backed by fixtures that normalize successful, partial, missing-field, stale, and changed-format outputs.

## Prerequisites

- [ ] Directive 003: Core Domain Models and Provider Contract - Complete

## References

**PRD:**
- User Story: US-002 Capture Codex Usage Locally
- User Story: US-010 Preserve Prototype Compatibility Where Practical
- Functional Requirements: FR-1, FR-4, FR-5, FR-6, FR-7, FR-27, FR-30
- Feature Specification: Codex Provider and Snapshot Contract

**ARCH.md:**
- Data Models: ProviderSnapshot, LimitWindow, ProviderMetadata, ProviderReadResult, AppError
- API Contracts: Rust Provider Trait `UsageProvider::refresh` data output contract
- Directory Structure: `providers/codex/`, `providers/codex/fixtures/`, `tests/fixtures/`, `local/`
- Error Codes: `PARSER_FAILED`, `PARTIAL_SNAPSHOT`, `CODEX_UNAUTHENTICATED`, `CAPTURE_FAILED`

**RESEARCH.md:**
- Patterns: Pattern 6: Frontend-Safe Typed Errors With Suggestions
- Anti-patterns: Anti-Pattern 3: Webview-Owned Shell Execution; Anti-Pattern 5: Product Code in the Prototype Folder
- Libraries: serde_json, thiserror

## Scope

### In Scope

- Read `local/monitor.sh` and fixture behavior as reference only.
- Implement a Codex parser in `providers/codex/` that accepts sanitized raw output strings and returns normalized `ProviderSnapshot` or structured parser errors.
- Extract Codex 5-hour remaining percentage, weekly remaining percentage, reset text for both windows, scrape time, and safe non-secret metadata where available.
- Compute `used_pct` from `remaining_pct` when Codex reports remaining.
- Assign `healthy`, `watch`, `low`, and `critical` statuses based on ARCH.md thresholds.
- Return partial snapshots when one valid limit exists and the missing limit is represented by `PARTIAL_SNAPSHOT`.
- Add fixtures for successful output, missing 5-hour field, missing weekly field, unauthenticated output, stale/prototype data shape, and changed-format parser failure.
- Add a compatibility test that either preserves `local/data.json` and `local/history.json` shape where practical or documents intentionally non-blocking differences.

### Out of Scope

- Running Codex, WSL, PTY, or any shell command.
- Writing latest snapshots or history.
- Implementing polling, Tauri commands, widget UI, tray, alerts, or settings.
- Modifying `local/monitor.sh` or turning `local/` into product code.

## Acceptance Criteria

- [ ] Parser tests pass for successful Codex output with exactly two limits: `5h` and `weekly`.
- [ ] Parser extracts 5-hour `remaining_pct`, weekly `remaining_pct`, and reset text when present.
- [ ] Parser computes `used_pct` so `remaining_pct + used_pct = 100` for parsed percentage values.
- [ ] Codex-specific raw fields appear only under provider-specific metadata and contain no credentials or webhook URLs.
- [ ] Missing one expected limit returns a `partial` read result or parse output with `PARTIAL_SNAPSHOT` context.
- [ ] Missing both expected limits returns `PARSER_FAILED` unless a known authentication failure applies.
- [ ] Unauthenticated output maps to `CODEX_UNAUTHENTICATED` with a short actionable message.
- [ ] Compatibility tests prove useful prototype parser behavior is reused or covered by equivalent fixtures.
- [ ] Existing `local/data.json` and `local/history.json` are not changed by this directive.
- [ ] All new code has corresponding tests in `tests/`, crate-level `tests/`, or `apps/desktop/src/__tests__/`.
- [ ] `cargo check --workspace` passes.
- [ ] `cargo test --workspace` passes.
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` passes.
- [ ] `cargo fmt --all --check` passes.

## Implementation Notes

- Avoid ad hoc parsing spread across the app. Keep all raw Codex output parsing inside `providers/codex`.
- Prefer tolerant matching for labels and whitespace, but fail loudly with `PARSER_FAILED` when both required windows cannot be interpreted.
- Sanitized fixtures must not include real account identifiers, raw terminal transcripts with sensitive details, webhook URLs, or tokens.

## Status: [ ] Incomplete / [x] Complete

## Notes

- Ported useful prototype parsing behavior into `providers/codex`, including tolerant `5h limit` and `weekly limit` matching, reset text extraction, used percentage calculation, status assignment, and partial snapshots.
- Added sanitized fixtures for success, missing 5h, missing weekly, unauthenticated output, changed format, and prototype-shaped output.
- Existing `local/data.json`, `local/history.json`, `local/monitor.sh`, and `local/dashboard.html` were not modified.
