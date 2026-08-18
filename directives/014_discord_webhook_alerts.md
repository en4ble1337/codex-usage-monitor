# Directive 014: Discord Webhook Alerts

## Objective

Add the Discord alert channel and `test_discord_webhook` command so configured webhook alerts are delivered from Rust without exposing secrets to the React webview.

## Prerequisites

- [ ] Directive 013: Alert Thresholds and Native Notifications - Complete

## References

**PRD:**
- User Story: US-006 Alert on Low Remaining Usage
- Functional Requirements: FR-9, FR-21, FR-22, FR-23
- Feature Specification: Alerts; Configuration

**ARCH.md:**
- Data Models: AppConfig, AlertState, AlertStateEntry, AppError
- API Contracts: `test_discord_webhook`
- Directory Structure: `core/ida-core/src/alerts/`, `apps/desktop/src-tauri/`
- Error Codes: `DISCORD_NOT_CONFIGURED`, `DISCORD_WEBHOOK_INVALID`, `DISCORD_DELIVERY_FAILED`, `ALERT_STATE_WRITE_FAILED`, `INTERNAL_ERROR`

**RESEARCH.md:**
- Patterns: Pattern 5: Rust-Side Webhook Delivery; Pattern 6: Frontend-Safe Typed Errors With Suggestions
- Anti-patterns: Anti-Pattern 4: Secrets in General App State
- Libraries: reqwest 0.12.x, serde_json, tracing

## Scope

### In Scope

- Implement Discord webhook URL validation and delivery in Rust.
- Add Discord as an alert channel using the provider-neutral alert boundary from Directive 013.
- Include provider, limit window, remaining percentage, threshold, and reset time in Discord messages.
- Deduplicate Discord alerts with the same provider/limit/threshold/channel/reset window key.
- Implement the `test_discord_webhook` Tauri command for configured or supplied webhook URLs.
- Ensure `get_config` and settings-facing data expose only secret presence or masked values.
- Add tests for valid Discord URLs, invalid URLs, not-configured command, successful mock delivery, failed delivery, dedupe, and secret redaction.

### Out of Scope

- Settings UI controls for Discord webhook entry.
- Telegram alerts.
- Sending Discord webhooks from the webview.
- Retrying failed Discord delivery beyond recording failure.

## Acceptance Criteria

- [ ] Discord alerts fire when `discord_alerts_enabled = true` and a valid webhook is configured.
- [ ] Discord delivery happens from Rust, not from React or direct browser network calls.
- [ ] `test_discord_webhook` sends a test alert using a supplied unsaved URL or the configured secret.
- [ ] `test_discord_webhook` returns `DISCORD_NOT_CONFIGURED` when no URL is available.
- [ ] Malformed or non-Discord webhook URLs return `DISCORD_WEBHOOK_INVALID`.
- [ ] Non-2xx Discord responses or network failures return `DISCORD_DELIVERY_FAILED`.
- [ ] Discord alerts are deduplicated independently from native notifications by channel.
- [ ] No Discord webhook URL appears in snapshots, history, alert state, generated bindings, logs, test fixtures, or UI state dumps.
- [ ] Tests mock HTTP delivery and do not require real Discord or network access.
- [ ] All new code has corresponding tests in `tests/`, crate-level `tests/`, or `apps/desktop/src/__tests__/`.
- [ ] `cargo check --workspace` passes.
- [ ] `cargo test --workspace` passes.
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` passes.
- [ ] `pnpm --dir apps/desktop typecheck` passes if command bindings are regenerated.
- [ ] `pnpm --dir apps/desktop test` passes if command wrappers are touched.
- [ ] `cargo fmt --all --check` and `pnpm --dir apps/desktop format:check` pass if frontend files are touched.

## Implementation Notes

- The alert payload can be simple text or a small JSON embed, but it must not include raw provider output.
- Use `reqwest` with bounded timeout.
- Redact webhook URLs before logging errors. Logging the host `discord.com` is fine; logging the token path is not.

## Status: [ ] Incomplete / [x] Complete

## Notes

- Implemented Discord webhook validation, Rust-side reqwest delivery, configured-webhook alert channel, per-channel dedupe, and `test_discord_webhook`.
- Settings-facing config exposes only secret presence/masked placeholder; tests use mock transports and no real network or webhook secret.
- Discord webhook URLs are not stored in snapshots, history, alert state, generated bindings, fixtures, logs, or UI state dumps.
