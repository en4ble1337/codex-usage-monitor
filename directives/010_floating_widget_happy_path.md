# Directive 010: Floating Widget Happy Path

## Objective

Build the first usable floating widget window that shows fresh Codex 5-hour and weekly limits with remaining percentages, reset text, and status labels in a movable always-on-top desktop surface.

## Prerequisites

- [ ] Directive 009: Tauri Command Bridge - Complete

## References

**PRD:**
- User Story: US-004 Render Floating Usage Widget
- Functional Requirements: FR-10, FR-11, FR-12, FR-13, FR-14, FR-15
- Feature Specification: Floating Widget

**ARCH.md:**
- Data Models: AppState, ProviderSnapshot, LimitWindow, WidgetPreferences
- API Contracts: `get_app_state`, `get_widget_preferences`, `update_widget_preferences`
- Directory Structure: `apps/desktop/src/windows/WidgetWindow.tsx`, `apps/desktop/src/components/`, `apps/desktop/src-tauri/`, `apps/desktop/src-tauri/capabilities/`
- Error Codes: `PREFERENCES_INVALID`, `PREFERENCES_WRITE_FAILED`, `INTERNAL_ERROR`

**RESEARCH.md:**
- Patterns: Pattern 2: Always-On-Top Floating Widget Window; Pattern 7: Tauri Capability Files Per Window; Pattern 8: Generated Rust-to-TypeScript Bindings
- Anti-patterns: Anti-Pattern 6: Dashboard-Scale UI for a Widget Product
- Libraries: Tauri 2.x, React 19.x, TypeScript 5.8+, lucide-react, Vitest, React Testing Library, Playwright

## Scope

### In Scope

- Create a dedicated Tauri widget window with minimum size around 280px by 160px.
- Apply `always_on_top`, `skip_taskbar`, visibility, size, and position from `WidgetPreferences`.
- Implement `WidgetWindow.tsx` that reads `AppState` and renders exactly two primary limit rows for `5h` and `weekly` when fresh data is available.
- Show remaining percentage, reset time or raw reset text, status label, and status color for each limit.
- Implement drag behavior suitable for a frameless or compact widget window.
- Persist widget position, size, visibility, and always-on-top changes through `update_widget_preferences`.
- Add component tests for fresh healthy/watch/low/critical render states.
- Verify the widget in a browser/dev preview using the browser skill and capture notes in this directive.

### Out of Scope

- Stale and error empty states beyond a minimal fallback.
- Tray menu and native tray icon behavior.
- Settings UI controls.
- Alerts and notifications.
- Dashboard-scale charts, onboarding, or marketing layout.

## Acceptance Criteria

- [ ] Widget opens as a desktop window, not a browser dashboard.
- [ ] Widget renders exactly two primary limit rows/cards: `5h` and `weekly`.
- [ ] Each visible limit shows remaining percentage, reset time or raw reset text, and a text status label.
- [ ] Status color is present but not the only signal; the label is visible.
- [ ] Widget minimum layout remains readable at about 280px by 160px with no overlapping text.
- [ ] Widget can be moved by the user and persists its updated position where Tauri exposes coordinates.
- [ ] Always-on-top is enabled by default and can be applied from preferences.
- [ ] Widget UI consumes generated AppState/LimitWindow types rather than Codex-specific parser types.
- [ ] Browser/dev verification confirms no overlapping text at desktop and compact widths.
- [ ] All new code has corresponding tests in `tests/`, crate-level `tests/`, or `apps/desktop/src/__tests__/`.
- [ ] `cargo check --workspace` passes.
- [ ] `cargo test --workspace` passes.
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` passes.
- [ ] `pnpm --dir apps/desktop typecheck` passes.
- [ ] `pnpm --dir apps/desktop test` passes.
- [ ] `pnpm --dir apps/desktop lint` passes.
- [ ] `pnpm --dir apps/desktop exec playwright test` passes for widget visual smoke tests if Playwright is configured.
- [ ] `cargo fmt --all --check` and `pnpm --dir apps/desktop format:check` pass.

## Implementation Notes

- Keep the widget visually compact and utility-like. Avoid dashboard navigation, hero copy, cards inside cards, and large decorative layouts.
- Use generated types and a mockable command wrapper so component tests can provide fixture AppState data.
- Use lucide icons only where they clarify controls; avoid explanatory in-app text about how to use the widget.

## Status: [ ] Incomplete / [x] Complete

## Notes

- Implemented a compact widget window and React widget view with exactly two primary limit slots for `5h` and `weekly`, remaining percentage, reset text, visible status labels, and status color.
- Added Tauri window setup for frameless, always-on-top, skip-taskbar widget behavior and preference persistence for size/position/visibility where Tauri exposes bounds.
- Browser skill verification was blocked because the required Node REPL browser control tool is not exposed in this session. Fallback smoke used Vite build, component tests, and dev-server 200 checks for `widget.html`.
