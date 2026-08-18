# Directive 016: Distribution Documentation and Release Smoke

## Objective

Document the open-source run/build path and perform final smoke verification so a technical Windows user can build Ida locally while understanding prototype boundaries and future paid convenience builds.

## Prerequisites

- [ ] Directive 015: Settings Window and Configuration UX - Complete

## References

**PRD:**
- User Story: US-000 Create Non-Colliding Product Structure
- User Story: US-009 Document Open-Source Build Path
- User Story: US-010 Preserve Prototype Compatibility Where Practical
- Functional Requirements: FR-2, FR-3, FR-28, FR-29, FR-30, FR-31
- Feature Specification: Repository Structure; Distribution Model

**ARCH.md:**
- Data Models: None
- API Contracts: All implemented commands are smoke-tested, with no new endpoint ownership in this directive
- Directory Structure: `README.md`, `docs/`, `execution/`, `.github/` if added, `local/`
- Error Codes: None

**RESEARCH.md:**
- Patterns: Pattern 1: Tray-Owned Utility Lifecycle; Pattern 2: Always-On-Top Floating Widget Window; Pattern 7: Tauri Capability Files Per Window
- Anti-patterns: Anti-Pattern 1: Broad App Architecture by Copying a Large Product; Anti-Pattern 5: Product Code in the Prototype Folder
- Libraries: Tauri bundler 2.x, pnpm 10.x, Rust 1.82+ stable, Node 24 LTS

## Scope

### In Scope

- Update README and/or docs with Ida MVP Windows setup instructions.
- Document prerequisites: Rust 1.82+ stable, Node 24 LTS preferred or Node 22.12+ minimum, pnpm 10.x, Tauri prerequisites, Codex authentication, and WSL fallback requirements.
- Document how to run from source with `pnpm --dir apps/desktop tauri dev`.
- Document how to build a local package with `pnpm --dir apps/desktop tauri build` if packaging is wired.
- Document that `local/` remains the Codex Limits prototype/reference implementation.
- Document the folder responsibilities for app, core, providers, docs, directives, execution scripts, tests, and prototype code.
- Document that paid convenience builds are future packaging/distribution convenience and no payment, license, account, or cloud backend exists in the MVP.
- Add an execution smoke checklist or script that runs format, lint, typecheck, Rust tests, frontend tests, and build checks.
- Perform final smoke verification for widget, tray, refresh, stale/error state, settings persistence, alert dedupe, Discord test command with mock or dry-run, and local packaging if available.

### Out of Scope

- Adding payment processing, license enforcement, auto-update, signing, or hosted accounts.
- Shipping macOS/Linux installers.
- Removing or rewriting the existing `local/` prototype.
- Implementing any new runtime feature beyond docs and smoke fixes needed to make the existing MVP coherent.

## Acceptance Criteria

- [ ] README or docs explain the Windows MVP setup path.
- [ ] Docs list required local dependencies, including Codex authentication and WSL if required for the user's setup.
- [ ] Docs explain how to run from source with the actual project command.
- [ ] Docs explain how to build a local app package if packaging exists.
- [ ] Docs clarify that source builds are open source and future paid builds are convenience builds only.
- [ ] Docs clearly mark `local/` as prototype/reference code and new Ida code as product code.
- [ ] Docs describe folder responsibilities for app, core, providers, docs, directives, execution scripts, tests, and prototype.
- [ ] Existing `local/monitor.sh`, `local/dashboard.html`, `local/data.json`, and `local/history.json` still work or any known prototype incompatibility is explicitly documented.
- [ ] Final verification includes widget happy path, stale/error state, tray controls, settings persistence, native alert behavior with mocks or local smoke, Discord test command with mock/dry-run, and source build.
- [ ] All new docs or scripts have corresponding checks where practical.
- [ ] `cargo check --workspace` passes.
- [ ] `cargo test --workspace` passes.
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` passes.
- [ ] `pnpm --dir apps/desktop typecheck` passes.
- [ ] `pnpm --dir apps/desktop test` passes.
- [ ] `pnpm --dir apps/desktop lint` passes.
- [ ] `pnpm --dir apps/desktop exec playwright test` passes if Playwright is configured.
- [ ] `pnpm --dir apps/desktop tauri build` passes if packaging is wired for this environment; otherwise the blocker is documented.
- [ ] `cargo fmt --all --check` and `pnpm --dir apps/desktop format:check` pass.

## Implementation Notes

- Keep docs practical and command-oriented. Avoid promising paid builds, auto-update, or signing as MVP features.
- Do not document Telegram as an MVP alert channel.
- Make Windows-first assumptions explicit while keeping future macOS/Linux paths described as future work.

## Status: [ ] Incomplete / [x] Complete

## Notes

- Updated `README.md` and added `docs/SMOKE.md` with Windows MVP prerequisites, run/build commands, folder responsibilities, prototype boundary, and source/convenience-build distribution notes.
- Added `execution/smoke.ps1` to run Rust checks, frontend checks, and optional Tauri build.
- Final package smoke succeeded: `corepack pnpm --dir apps/desktop tauri build` produced `target/release/bundle/nsis/Ida_0.1.0_x64-setup.exe`.
- `pnpm` is not globally installed in this shell, so verification used `corepack pnpm`; the docs include both forms.
