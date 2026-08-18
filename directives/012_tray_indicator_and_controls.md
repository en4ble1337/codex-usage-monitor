# Directive 012: Tray Indicator and Controls

## Objective

Add the Windows tray indicator and tray-owned lifecycle controls so Ida can run quietly, expose widget controls, trigger manual refresh, open configuration, and quit cleanly.

## Prerequisites

- [ ] Directive 011: Stale and Error Widget States - Complete

## References

**PRD:**
- User Story: US-005 Provide Tray Indicator and Controls
- Functional Requirements: FR-2, FR-16, FR-17, FR-18, FR-19
- Feature Specification: Tray Indicator

**ARCH.md:**
- Data Models: AppState, WidgetPreferences
- API Contracts: `refresh_usage`, `update_widget_preferences`, `open_config_directory`, `quit_app`
- Directory Structure: `apps/desktop/src-tauri/`, `apps/desktop/src-tauri/icons/`, `apps/desktop/src/windows/SettingsWindow.tsx`
- Error Codes: `PREFERENCES_WRITE_FAILED`, `PROVIDER_NOT_FOUND`, `CODEX_NOT_FOUND`, `INTERNAL_ERROR`

**RESEARCH.md:**
- Patterns: Pattern 1: Tray-Owned Utility Lifecycle; Pattern 7: Tauri Capability Files Per Window
- Anti-patterns: Anti-Pattern 6: Dashboard-Scale UI for a Widget Product
- Libraries: Tauri tray/menu APIs, tauri-plugin-single-instance, lucide-react

## Scope

### In Scope

- Create a native tray icon while Ida is running.
- Add menu items for Show Widget, Hide Widget, Refresh Now, Settings or Configure, and Quit.
- Toggle widget visibility from tray commands and persist visibility preferences.
- Trigger `refresh_usage` from Refresh Now and update widget/tray state after completion.
- Implement Quit so polling stops and the app exits without leaving duplicate tray icons.
- Derive tray tooltip/menu label or icon state from the most severe current status where technically feasible.
- Add a minimal settings/configure window shell or open the config directory from the Settings menu until full settings UI arrives.
- Add Rust tests for tray command dispatch handlers where practical and manual Windows smoke notes.

### Out of Scope

- Full editable settings UI.
- Alert delivery and notification threshold behavior.
- Advanced custom tray art beyond basic status icons.
- macOS menu bar or Linux AppIndicator hardening.

## Acceptance Criteria

- [ ] App creates a Windows tray icon while running.
- [ ] Tray menu contains Show Widget or Hide Widget, Refresh Now, Settings or Configure, and Quit.
- [ ] Show Widget makes the widget visible and focused.
- [ ] Hide Widget hides the widget while keeping the app and polling alive.
- [ ] Refresh Now starts a manual provider refresh and emits or applies updated state after completion.
- [ ] Quit stops polling and exits the app process.
- [ ] Tray visual state, tooltip, or menu label reflects the most severe current status when Tauri APIs make it feasible.
- [ ] Closing the widget window hides it instead of quitting unless the user selects Quit.
- [ ] A second app launch does not create a duplicate tray icon or polling loop.
- [ ] Windows smoke testing notes are added to this directive's Notes section during implementation.
- [ ] All new code has corresponding tests in `tests/`, crate-level `tests/`, or `apps/desktop/src/__tests__/`.
- [ ] `cargo check --workspace` passes.
- [ ] `cargo test --workspace` passes.
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` passes.
- [ ] `pnpm --dir apps/desktop typecheck` passes.
- [ ] `pnpm --dir apps/desktop test` passes.
- [ ] `pnpm --dir apps/desktop lint` passes.
- [ ] `cargo fmt --all --check` and `pnpm --dir apps/desktop format:check` pass.

## Implementation Notes

- Follow official Tauri v2 tray APIs when examples disagree.
- Keep tray event handlers thin and route work through the same command/runtime services as the UI.
- The Settings item may open a minimal window shell here; Directive 015 fills in the configuration controls.

## Status: [ ] Incomplete / [x] Complete

## Notes

- Added native tray creation with Show Widget, Hide Widget, Refresh Now, Settings, and Quit menu items.
- Widget close requests hide the widget while keeping the app alive; tray refresh routes through the same runtime and emits `ida:state-changed`.
- Windows package smoke built successfully with Tauri/NSIS. Interactive tray clicking was not manually driven in this non-interactive session.
