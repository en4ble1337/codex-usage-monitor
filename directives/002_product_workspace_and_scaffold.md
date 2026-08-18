# Directive 002: Product Workspace and Scaffold

## Objective

Create the non-colliding Ida product structure and minimal build scaffold so new desktop, core, and provider code can be added without touching the existing `local/` prototype.

## Prerequisites

- [ ] Directive 001: Initial Setup - Complete

## References

**PRD:**
- User Story: US-000 Create Non-Colliding Product Structure
- Functional Requirements: FR-2, FR-29, FR-30, FR-31
- Feature Specification: Repository Structure

**ARCH.md:**
- Data Models: None
- API Contracts: None
- Directory Structure: `apps/desktop/`, `apps/desktop/src/`, `apps/desktop/src-tauri/`, `core/ida-core/`, `providers/codex/`, `providers/claude/`, `tests/`, `execution/`, `.tmp/`
- Error Codes: None

**RESEARCH.md:**
- Patterns: Pattern 7: Tauri Capability Files Per Window; Pattern 8: Generated Rust-to-TypeScript Bindings
- Anti-patterns: Anti-Pattern 1: Broad App Architecture by Copying a Large Product; Anti-Pattern 5: Product Code in the Prototype Folder
- Libraries: Tauri 2.x, React 19.x, TypeScript 5.8+, Vite 7.x, pnpm 10.x, Rust 1.82+ stable, specta

## Scope

### In Scope

- Create the top-level Ida product folders described in ARCH.md.
- Initialize a Rust workspace rooted at the repository root with placeholder crates for `core/ida-core`, `providers/codex`, and `apps/desktop/src-tauri`.
- Initialize a pnpm workspace with a minimal React/Vite/TypeScript app under `apps/desktop/`.
- Add Tauri v2 configuration files and capability placeholders without implementing product commands yet.
- Add `providers/claude/README.md` as a future-provider placeholder only.
- Add or update documentation that explains `local/` is prototype/reference code and new Ida code lives outside it.
- Add `.gitignore` entries for `target/`, `node_modules/`, Tauri build output, `.tmp/`, and local app/config artifacts.

### Out of Scope

- Implementing domain models, parser logic, storage, Tauri commands, widget UI, tray behavior, alerts, or settings forms.
- Modifying `local/monitor.sh`, `local/dashboard.html`, `local/data.json`, or `local/history.json`.
- Adding Claude provider implementation beyond a placeholder README.
- Adding CI workflows or release packaging.

## Acceptance Criteria

- [ ] `apps/desktop/`, `core/ida-core/`, `providers/codex/`, `providers/claude/`, `tests/`, `execution/`, and `.tmp/` exist with appropriate placeholder or scaffold files.
- [ ] The existing `local/` prototype files remain unchanged by this directive.
- [ ] Documentation states which folders are prototype, app, core, provider, docs, directives, test, execution, and scratch areas.
- [ ] The Rust workspace includes `core/ida-core`, `providers/codex`, and `apps/desktop/src-tauri` members.
- [ ] The frontend workspace uses pnpm and places React/Vite source under `apps/desktop/src/`.
- [ ] No build outputs, generated installers, `node_modules`, `target`, or local secret/config files are committed.
- [ ] All new code has corresponding smoke tests or scaffold verification in `tests/`, crate-level `tests/`, or `apps/desktop/src/__tests__/`.
- [ ] `cargo check --workspace` passes.
- [ ] `cargo test --workspace` passes.
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` passes.
- [ ] `pnpm --dir apps/desktop typecheck` passes.
- [ ] `pnpm --dir apps/desktop test` passes.
- [ ] `pnpm --dir apps/desktop lint` passes.
- [ ] `cargo fmt --all --check` and `pnpm --dir apps/desktop format:check` pass.

## Implementation Notes

- Keep the scaffold small. Do not import a large app template wholesale.
- Use Node 24 LTS where available and allow Node 22.12+ as the documented minimum.
- The first Tauri capability files should be narrow placeholders; the widget must not get shell, filesystem, or network permissions.
- Treat `local/` as read-only reference material unless a later directive explicitly says otherwise.

## Status: [ ] Incomplete / [x] Complete

## Notes

- Created the Ida product workspace outside `local/`: Rust workspace, pnpm workspace, Tauri v2 app shell, React/Vite app, core/provider crates, tests, execution, and scratch folders.
- Added narrow widget/settings capability placeholders and a generated app icon for Tauri build resources.
- Documented folder responsibilities and the `local/` prototype boundary in `README.md`.
- `pnpm` was not globally available in this environment; Corepack pnpm 10.20.0 was used for verification, and Tauri before-build commands use `corepack pnpm`.
