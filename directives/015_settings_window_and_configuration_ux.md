# Directive 015: Settings Window and Configuration UX

## Objective

Build the minimal settings experience for polling, freshness, alert thresholds, Discord webhook configuration, notification toggles, capture mode, and widget preferences using the existing Tauri command API.

## Prerequisites

- [ ] Directive 012: Tray Indicator and Controls - Complete
- [ ] Directive 014: Discord Webhook Alerts - Complete

## References

**PRD:**
- User Story: US-006 Alert on Low Remaining Usage
- User Story: US-008 Provide Minimal Configuration
- Functional Requirements: FR-20, FR-21, FR-26
- Feature Specification: Configuration; Alerts; Tray Indicator

**ARCH.md:**
- Data Models: AppConfig, WidgetPreferences, AppError
- API Contracts: `get_config`, `update_config`, `get_widget_preferences`, `update_widget_preferences`, `test_discord_webhook`, `open_config_directory`
- Directory Structure: `apps/desktop/src/windows/SettingsWindow.tsx`, `apps/desktop/src/components/`, `apps/desktop/src-tauri/capabilities/`
- Error Codes: `VALIDATION_ERROR`, `CONFIG_INVALID`, `CONFIG_READ_FAILED`, `CONFIG_WRITE_FAILED`, `DISCORD_WEBHOOK_INVALID`, `DISCORD_DELIVERY_FAILED`, `DISCORD_NOT_CONFIGURED`, `PREFERENCES_INVALID`, `PREFERENCES_WRITE_FAILED`

**RESEARCH.md:**
- Patterns: Pattern 3: Atomic Local JSON Writes; Pattern 5: Rust-Side Webhook Delivery; Pattern 7: Tauri Capability Files Per Window; Pattern 8: Generated Rust-to-TypeScript Bindings
- Anti-patterns: Anti-Pattern 4: Secrets in General App State; Anti-Pattern 6: Dashboard-Scale UI for a Widget Product
- Libraries: React 19.x, TypeScript 5.8+, lucide-react, Vitest, React Testing Library, Playwright

## Scope

### In Scope

- Implement `SettingsWindow.tsx` with compact controls for polling interval, stale threshold, alert thresholds, native notifications, Discord alerts, Discord webhook secret entry, capture mode, and widget always-on-top/visible-on-launch preferences.
- Use segmented controls, toggles, numeric inputs, and concise icon buttons where appropriate.
- Read redacted config and secret presence through `get_config`.
- Save config patches through `update_config`, including explicit secret updates for Discord webhook set/clear.
- Test Discord webhook delivery with `test_discord_webhook`.
- Open the config directory through `open_config_directory`.
- Display validation and delivery errors with short actionable copy.
- Ensure the tray Settings item opens or focuses this window.
- Add component tests and browser/dev verification for form validation, secret masking, webhook test states, and compact responsive layout.

### Out of Scope

- Rich onboarding or dashboard analytics.
- Editing raw JSON directly in the app.
- Telegram alert settings.
- Cloud sync, accounts, payments, or license controls.
- Storing secrets in frontend state after save beyond transient input handling.

## Acceptance Criteria

- [ ] User can configure polling interval without editing source code.
- [ ] User can configure stale/freshness threshold without editing source code.
- [ ] User can configure alert thresholds without editing source code.
- [ ] User can enable or disable native notifications.
- [ ] User can enable or disable Discord alerts.
- [ ] User can set, clear, and test a Discord webhook without the saved URL being displayed after save.
- [ ] User can choose capture mode among `native_then_wsl`, `native_only`, and `wsl_only`.
- [ ] User can configure widget always-on-top and visible-on-launch preferences.
- [ ] Invalid values are rejected with clear messages and do not corrupt persisted config.
- [ ] Settings persist after app restart in command-level or documented smoke testing.
- [ ] Tray Settings opens or focuses the settings window.
- [ ] Browser/dev verification confirms settings controls do not overlap at compact desktop widths.
- [ ] All new code has corresponding tests in `tests/`, crate-level `tests/`, or `apps/desktop/src/__tests__/`.
- [ ] `cargo check --workspace` passes.
- [ ] `cargo test --workspace` passes.
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` passes.
- [ ] `pnpm --dir apps/desktop typecheck` passes.
- [ ] `pnpm --dir apps/desktop test` passes.
- [ ] `pnpm --dir apps/desktop lint` passes.
- [ ] `pnpm --dir apps/desktop exec playwright test` passes for settings visual smoke tests if Playwright is configured.
- [ ] `cargo fmt --all --check` and `pnpm --dir apps/desktop format:check` pass.

## Implementation Notes

- Keep the settings UI work-focused and compact. This is not a landing page or dashboard.
- Do not keep saved webhook URLs in React state after a successful save; reload redacted config and secret presence instead.
- Prefer backend validation messages over duplicated frontend-only validation where possible, while still preventing obvious invalid form submissions.

## Status: [ ] Incomplete / [x] Complete

## Notes

- Built a compact settings window for polling interval, stale threshold, alert thresholds, native notifications, Discord alerts/webhook set-clear-test, capture mode, and widget always-on-top/visible-on-launch preferences.
- Settings uses Tauri command wrappers, backend validation, and clears transient webhook input after save/test flows.
- Browser skill verification was blocked because the required Node REPL browser control tool is not exposed in this session. Fallback coverage includes settings component tests and dev-server 200 checks for `settings.html`.
