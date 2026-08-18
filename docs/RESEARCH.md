# Ida Implementation Research

## Research Summary

**Search Date:** 2026-05-03
**Tech Stack Context:** Tauri v2 desktop app with Rust 1.82+, Tokio, serde/serde_json, React 19, TypeScript 5.8+, Vite 7, pnpm 10, Tauri tray/menu APIs, Tauri notification plugin, reqwest, tracing, local JSON/NDJSON persistence.
**Primary Search Terms:** Tauri v2 React TypeScript tray app, Tauri v2 system tray notifications Rust, Tauri always-on-top floating window, Rust Tauri local settings JSON, Rust Discord webhook reqwest.
**Repositories Evaluated:** 12
**Repositories Recommended:** 5

### Extracted Search Context

**From ARCH.md**

- **Tech stack:** Tauri v2, Rust, Tokio, React, TypeScript, Vite, pnpm, serde, reqwest, tracing, Tauri path/tray/notification APIs.
- **Domain terms:** Ida, Desktop App, Core, Provider, Codex Provider, Provider Snapshot, Limit Window, Latest Snapshot, Stale, Structured Error, Widget, Floating Widget, Tray Icon, Alert, Alert Deduplication, Discord Webhook, WSL.
- **Key patterns needed:** Tauri tray menu and click handlers, always-on-top floating window, Rust command layer, provider boundary, async polling loop, native/WSL process capture, JSON/NDJSON local storage with atomic writes, typed error mapping, notification delivery, Discord webhook delivery, settings persistence, frontend test mocks for Tauri APIs.

**From PRD.md**

- **Core feature keywords:** local-first desktop usage monitor, Codex usage limits, 5-hour and weekly limits, floating always-on-top widget, tray indicator, stale/error state, local latest snapshot, short history, threshold alerts, minimal configuration.
- **Integration points:** Codex CLI, WSL, native OS notifications, Discord webhook, local filesystem. Telegram is deferred to a future alert channel behind the generic alert-channel interface.

### Evaluated but Not Recommended

| Repository | Reason |
|------------|--------|
| `kitlib/tauri-app-template` | Compatible MIT template, but less directly useful than `dannysmith/tauri-template` and leans toward a heavier UI starter than ARCH.md's plain/scoped CSS preference. |
| `screenpipe/screenpipe` | Active and relevant local-first Tauri/Rust app, but GitHub reports license as `NOASSERTION`, so it fails the license filter. |
| `PasteBar/PasteBarApp` | Tray/local utility patterns are relevant, but GitHub reports license as `NOASSERTION`, so it fails the license filter. |
| `dongdongbh/Mindwtr` | Active local-first Tauri app, but AGPL-3.0 is incompatible with the approved filter. |
| `lemotw/vedrr` | Interesting Tauri 2 local app, but GPL-3.0 and below the general star threshold. |
| `Glsme/agent-monitor` | Niche AI-monitoring surface, but no license was reported in the search result. |

## Recommended Repositories

### Repo 1: Rench321/sklad

- **URL:** https://github.com/Rench321/sklad
- **Stars:** 171 | **License:** MIT | **Last Updated:** 2026-04
- **Relevance:** High
- **Why Relevant:** Sklad is a real Tauri v2 + React tray-first desktop utility with local app-data storage, settings, native tray menu generation, notification usage, open-file helpers, and startup/quit behavior. It is the best practical reference for Ida's tray indicator and local utility ergonomics.

**Applicable Patterns:**

- [x] Directory structure approach
- [x] Tray menu and tray click handling
- [x] Tauri command registration
- [x] Local app-data settings/storage
- [x] Native notifications
- [ ] Testing patterns
- [ ] Provider/parser architecture

**Key Files to Study:**

| File | What to Learn |
|------|---------------|
| `src-tauri/src/lib.rs` | Tauri plugin setup, tray creation, menu events, left-click behavior, show/focus window helpers, and exit handling. |
| `src-tauri/src/tray_generator.rs` | Native menu construction from app state and menu regeneration after data changes. |
| `src-tauri/src/commands.rs` | Tauri command shape, notification delivery, settings save, opening app data/log directories, and tray menu refresh after state changes. |
| `src-tauri/src/data_manager.rs` | App data directory resolution, JSON file persistence, default data, backups, and backup rotation. |
| `src/components/Settings.tsx` | Compact settings UI patterns for a tray-first desktop utility. |

**Caveats:**

- No test suite was observed in the repository tree; do not borrow testing strategy from this repo.
- Its `DataManager` uses direct `fs::write` for core data, while Ida's ARCH.md requires atomic JSON writes for config/snapshot state.

### Repo 2: berbicanes/apiark

- **URL:** https://github.com/berbicanes/apiark
- **Stars:** 1,043 | **License:** MIT | **Last Updated:** 2026-05
- **Relevance:** High
- **Why Relevant:** APIArk is a privacy-first Tauri v2 local desktop app with a Rust backend, React frontend, structured command modules, local persistence, tests, async scheduled monitors, and webhook delivery. Its surface is much larger than Ida, but several backend patterns map directly to polling, settings, errors, and alert dispatch.

**Applicable Patterns:**

- [x] Directory structure approach
- [x] Tauri command module organization
- [x] Local settings persistence
- [x] Async polling/scheduler loop
- [x] Webhook delivery with reqwest
- [x] Error mapping
- [x] Frontend and Rust test placement
- [ ] Minimal dependency profile

**Key Files to Study:**

| File | What to Learn |
|------|---------------|
| `apps/desktop/src-tauri/src/lib.rs` | Large Tauri command registration, managed state setup, plugin setup, logging initialization, and recovery behavior. |
| `apps/desktop/src-tauri/src/storage/settings.rs` | Serde defaults and temp-file-then-rename atomic settings writes. |
| `apps/desktop/src-tauri/src/commands/settings.rs` | In-memory settings state guarded by a mutex plus persisted patch updates. |
| `apps/desktop/src-tauri/src/scheduler/monitor.rs` | Tokio task lifecycle, shutdown channel, scheduled loop, result events, and webhook-on-failure pattern. |
| `apps/desktop/src-tauri/src/models/error.rs` | Internal typed errors converted to frontend-safe error payloads with suggestions. |
| `apps/desktop/src-tauri/tests/integration_tests.rs` | Rust integration test placement for desktop backend behavior. |
| `apps/desktop/src/__tests__/mocks/tauri-api.ts` | Frontend mocks for Tauri APIs in Vitest. |

**Caveats:**

- APIArk is intentionally broad and includes SQLite, OAuth, proxying, plugins, AI features, licensing, and many network protocols. Ida should copy narrow patterns only, not the app architecture size.
- Some settings include secrets in a general settings struct. Ida should keep ARCH.md's stricter secret redaction and snapshot exclusion rules.

### Repo 3: dannysmith/tauri-template

- **URL:** https://github.com/dannysmith/tauri-template
- **Stars:** 243 | **License:** MIT | **Last Updated:** 2026-05
- **Relevance:** High
- **Why Relevant:** This is a production-oriented Tauri v2 + React 19 + TypeScript template with generated bindings, notification commands, quick floating pane patterns, window-state handling, plugin capability files, docs, and Vitest coverage. It aligns closely with Ida's planned stack and can inform Phase 3 scaffolding.

**Applicable Patterns:**

- [x] Directory structure approach
- [x] Always-on-top floating window
- [x] Native notification command
- [x] Settings/preferences persistence
- [x] Type generation with Specta
- [x] Tauri capability files per window
- [x] Frontend testing patterns
- [ ] Tray menu implementation

**Key Files to Study:**

| File | What to Learn |
|------|---------------|
| `src-tauri/src/lib.rs` | Plugin setup, generated command bindings, startup initialization, close/hide behavior, cleanup, and platform gating. |
| `src-tauri/src/commands/quick_pane.rs` | Hidden always-on-top floating window creation, show/hide/toggle commands, monitor-aware positioning, and shortcut registration. |
| `src-tauri/src/commands/preferences.rs` | App data directory resolution, input validation, JSON preferences, and atomic write cleanup. |
| `src-tauri/src/commands/notifications.rs` | Thin Rust-side wrapper around the Tauri notification plugin. |
| `src-tauri/capabilities/quick-pane.json` | Minimal permissions scoped to a secondary floating window. |
| `src-tauri/src/types.rs` | Serde + Specta model types and validation helpers. |
| `src/lib/commands/commands.test.ts` | Frontend command registry test pattern. |

**Caveats:**

- The template uses a broader UI/tooling stack than Ida needs. Keep ARCH.md's lightweight styling preference unless stakeholders approve the extra dependencies.
- Quick pane macOS-specific NSPanel handling is useful future context, but Windows MVP should keep the simpler standard Tauri window path.

### Repo 4: tauri-apps/tauri

- **URL:** https://github.com/tauri-apps/tauri
- **Stars:** 106,151 | **License:** Apache-2.0 | **Last Updated:** 2026-05
- **Relevance:** High
- **Why Relevant:** This is the canonical source for Tauri v2 behavior, examples, permissions, tray APIs, webview-window APIs, and Windows packaging internals. It should be treated as the authority when app examples disagree.

**Applicable Patterns:**

- [x] Tauri tray API reference implementation
- [x] Window and webview-window APIs
- [x] Capability/permission examples
- [x] Windows packaging reference
- [x] Cross-platform platform-gating examples
- [ ] Product app structure

**Key Files to Study:**

| File | What to Learn |
|------|---------------|
| `examples/api/src-tauri/src/tray.rs` | Official Tauri v2 tray creation, menu events, dynamic menu/icon updates, and show/focus behavior. |
| `crates/tauri/src/tray/mod.rs` | Tray API types and expected runtime behavior. |
| `crates/tauri/src/window/mod.rs` | Window API surface for always-on-top, focus, visibility, and position operations. |
| `crates/tauri/permissions/window/autogenerated/reference.md` | Capability names for window operations. |
| `crates/tauri/permissions/tray/autogenerated/reference.md` | Capability names for tray operations. |
| `crates/tauri-bundler/src/bundle/windows/` | Windows installer/bundler behavior for later distribution tasks. |

**Caveats:**

- It is framework source, not an app. Use it to settle API questions, not to shape Ida's product boundaries.

### Repo 5: tauri-apps/plugins-workspace

- **URL:** https://github.com/tauri-apps/plugins-workspace
- **Stars:** 1,714 | **License:** Apache-2.0 | **Last Updated:** 2026-05
- **Relevance:** Medium
- **Why Relevant:** This is the official Tauri v2 plugin workspace, including notification, shell, fs, opener, process, window-state, store, log, and updater plugin examples. Ida already plans to use notification APIs and may benefit from window-state, single-instance, process, or log plugin patterns after review.

**Applicable Patterns:**

- [x] Native notification plugin usage
- [x] Plugin permissions and capability examples
- [x] Store/window-state/process plugin references
- [x] Official examples for desktop plugin integration
- [ ] Ida app architecture
- [ ] Provider parser logic

**Key Files to Study:**

| File | What to Learn |
|------|---------------|
| `plugins/notification/README.md` | Current setup and usage details for native notifications. |
| `examples/api/src/views/Notifications.svelte` | End-to-end notification UI trigger pattern. |
| `examples/api/src-tauri/capabilities/desktop.json` | Desktop capability scoping across plugins. |
| `plugins/window-state/README.md` | Whether window-state can replace or complement custom widget preference persistence. |
| `plugins/process/README.md` | App-side process lifecycle helpers, if needed for quit/restart flows. |
| `plugins/shell/README.md` | Permission model reference; Ida should prefer Rust-side command execution for Codex capture. |

**Caveats:**

- Do not expose Codex/WSL shell execution to the webview just because a shell plugin exists. ARCH.md says provider command execution should live in Rust provider code and use argument arrays.

## Pattern Catalog

### Pattern 1: Tray-Owned Utility Lifecycle

**Source:** [Rench321/sklad](https://github.com/Rench321/sklad) - `src-tauri/src/lib.rs`, `src-tauri/src/tray_generator.rs`; [tauri-apps/tauri](https://github.com/tauri-apps/tauri) - `examples/api/src-tauri/src/tray.rs`
**Applies To:** US-005 Tray Indicator and Controls, ARCH.md Tray Icon, Tray Menu, Show Widget, Hide Widget, Refresh Now, Quit.

**Code Reference:**

```rust
// Pattern sketch derived from Sklad and Tauri official tray examples.
// Attribution: https://github.com/Rench321/sklad and https://github.com/tauri-apps/tauri
fn create_ida_tray(app: &tauri::AppHandle) -> tauri::Result<()> {
    let show_hide = MenuItem::with_id(app, "toggle-widget", "Show Widget", true, None::<&str>)?;
    let refresh = MenuItem::with_id(app, "refresh-now", "Refresh Now", true, None::<&str>)?;
    let settings = MenuItem::with_id(app, "settings", "Settings", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show_hide, &refresh, &settings, &quit])?;

    TrayIconBuilder::with_id("ida-main")
        .tooltip("Ida")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| handle_tray_command(app, event.id.as_ref()))
        .build(app)?;

    Ok(())
}
```

**Adaptation Notes:**

- Map menu IDs to ARCH.md commands: `Show Widget`, `Hide Widget`, `Refresh Now`, `Settings`, `Quit`.
- Keep tray state derived from `AppState.effective_limits`, especially the most severe `Limit Status`.
- Build a small `TrayController` in `apps/desktop/src-tauri/` that calls core services instead of importing Codex parser code.
- Use official Tauri examples for dynamic tray icon/menu updates when risk state changes.

### Pattern 2: Always-On-Top Floating Widget Window

**Source:** [dannysmith/tauri-template](https://github.com/dannysmith/tauri-template) - `src-tauri/src/commands/quick_pane.rs`
**Applies To:** US-004 Render Floating Usage Widget, ARCH.md Floating Widget, Widget Preferences, Minimum Widget Size.

**Code Reference:**

```rust
// Pattern sketch derived from dannysmith/tauri-template quick pane window.
// Attribution: https://github.com/dannysmith/tauri-template
fn create_widget_window(app: &AppHandle) -> Result<(), String> {
    WebviewWindowBuilder::new(app, "widget", WebviewUrl::App("widget.html".into()))
        .title("Ida")
        .inner_size(280.0, 160.0)
        .always_on_top(true)
        .skip_taskbar(true)
        .decorations(false)
        .resizable(false)
        .visible(false)
        .build()
        .map_err(|error| format!("Failed to create widget window: {error}"))?;

    Ok(())
}
```

**Adaptation Notes:**

- Rename `quick-pane` concepts to ARCH.md `Widget` and `Floating Widget`.
- Persist `WidgetPreferences` in Ida core, not only through a UI store.
- Use `always_on_top(true)`, `skip_taskbar(true)`, and minimum-size enforcement, but verify Windows drag behavior in Tauri.
- Prefer a dedicated `WidgetWindow.tsx` consuming generated `AppState` bindings.

### Pattern 3: Atomic Local JSON Writes

**Source:** [berbicanes/apiark](https://github.com/berbicanes/apiark) - `apps/desktop/src-tauri/src/storage/settings.rs`; [dannysmith/tauri-template](https://github.com/dannysmith/tauri-template) - `src-tauri/src/commands/preferences.rs`
**Applies To:** US-003 Store Latest Snapshot Locally, US-008 Provide Minimal Configuration, ARCH.md Snapshot Store, Config File, Widget Preferences.

**Code Reference:**

```rust
// Pattern sketch derived from APIArk and tauri-template settings persistence.
// Attribution: https://github.com/berbicanes/apiark and https://github.com/dannysmith/tauri-template
pub fn write_json_atomic<T: Serialize>(path: &Path, value: &T) -> Result<(), AppError> {
    let tmp_path = path.with_extension("tmp");
    let payload = serde_json::to_string_pretty(value)?;

    std::fs::create_dir_all(path.parent().ok_or(AppError::file_io("missing parent"))?)?;
    std::fs::write(&tmp_path, payload)?;
    std::fs::rename(&tmp_path, path)?;
    Ok(())
}
```

**Adaptation Notes:**

- Place this in `core/ida-core/src/storage/` and use it for `latest.json`, `config.json`, `alert-state.json`, and `widget-preferences.json`.
- Map failures to ARCH.md error codes: `CONFIG_WRITE_FAILED`, `SNAPSHOT_CORRUPT`, `PREFERENCES_WRITE_FAILED`, or `FILE_IO_ERROR`.
- Add tests that simulate invalid JSON and failed parent-directory creation.
- Do not follow Sklad's direct write approach for state that ARCH.md marks as atomic.

### Pattern 4: Async Polling Loop With Shutdown and Events

**Source:** [berbicanes/apiark](https://github.com/berbicanes/apiark) - `apps/desktop/src-tauri/src/scheduler/monitor.rs`
**Applies To:** US-002 Capture Codex Usage Locally, US-007 Handle Stale and Error States, ARCH.md Polling, ProviderReadResult, AppState.

**Code Reference:**

```rust
// Pattern sketch derived from APIArk monitor task lifecycle.
// Attribution: https://github.com/berbicanes/apiark
tokio::spawn(async move {
    loop {
        tokio::select! {
            _ = tokio::time::sleep(next_poll_delay(config.polling_interval_secs)) => {}
            _ = &mut shutdown_rx => break,
        }

        let result = provider.refresh(&runtime_config).await;
        app_state.apply_provider_result(result).await;
        let _ = app.emit("ida:state-changed", ());
    }
});
```

**Adaptation Notes:**

- Replace cron scheduling with Ida's simple `polling_interval_secs` from `AppConfig`.
- Store success in `Latest Snapshot` and `History`; failed scrapes update `current_error` without replacing the latest success.
- Emit one frontend event such as `ida:state-changed`; React can call `get_app_state`.
- Deduplicate manual refresh and scheduled refresh so `Refresh Now` cannot create overlapping Codex captures.

### Pattern 5: Rust-Side Webhook Delivery

**Source:** [berbicanes/apiark](https://github.com/berbicanes/apiark) - `apps/desktop/src-tauri/src/scheduler/monitor.rs`
**Applies To:** US-006 Alert on Low Remaining Usage, ARCH.md Discord Webhook, Alert Deduplication, Secret safety.

**Code Reference:**

```rust
// Pattern sketch derived from APIArk's webhook-on-failure dispatch.
// Attribution: https://github.com/berbicanes/apiark
async fn send_discord_alert(webhook_url: &str, alert: &AlertMessage) -> Result<(), AppError> {
    reqwest::Client::new()
        .post(webhook_url)
        .json(alert)
        .send()
        .await
        .map_err(AppError::discord_delivery_failed)?
        .error_for_status()
        .map_err(AppError::discord_delivery_failed)?;

    Ok(())
}
```

**Adaptation Notes:**

- Keep Discord delivery in Rust only; never expose webhook secrets to React.
- Validate webhook URLs before save and before send, returning `DISCORD_WEBHOOK_INVALID` when needed.
- Deduplicate by `provider_id`, `limit_id`, `threshold`, `channel`, and `Reset Window Key` before sending.
- Telegram should remain behind the same alert-channel interface if revived after MVP.

### Pattern 6: Frontend-Safe Typed Errors With Suggestions

**Source:** [berbicanes/apiark](https://github.com/berbicanes/apiark) - `apps/desktop/src-tauri/src/models/error.rs`; [dannysmith/tauri-template](https://github.com/dannysmith/tauri-template) - `src-tauri/src/types.rs`
**Applies To:** US-007 Handle Stale and Error States, ARCH.md Structured Error, Error Codes, Tauri Command Errors.

**Code Reference:**

```rust
// Pattern sketch derived from APIArk's internal-to-frontend error mapping.
// Attribution: https://github.com/berbicanes/apiark
impl From<CodexCaptureError> for AppError {
    fn from(error: CodexCaptureError) -> Self {
        match error {
            CodexCaptureError::NotFound => AppError::new("CODEX_NOT_FOUND")
                .message("Codex CLI was not found. Install Codex or update PATH, then refresh.")
                .retryable(true),
            CodexCaptureError::Unauthenticated => AppError::new("CODEX_UNAUTHENTICATED")
                .message("Codex is installed but not signed in. Open Codex once and sign in.")
                .retryable(true),
        }
    }
}
```

**Adaptation Notes:**

- Use ARCH.md's exact error codes and JSON shape.
- Keep provider-specific raw failure details inside internal logs; UI gets short, actionable messages.
- Add parser fixtures for `CODEX_NOT_FOUND`, `CODEX_UNAUTHENTICATED`, `WSL_NOT_FOUND`, `WSL_UNAVAILABLE`, `PARSER_FAILED`, and `PARTIAL_SNAPSHOT`.

### Pattern 7: Tauri Capability Files Per Window

**Source:** [dannysmith/tauri-template](https://github.com/dannysmith/tauri-template) - `src-tauri/capabilities/quick-pane.json`; [tauri-apps/tauri](https://github.com/tauri-apps/tauri) - permissions reference files
**Applies To:** ARCH.md CORS and WebView Policy, Widget Window, Settings Window.

**Code Reference:**

```json
{
  "identifier": "ida-widget-capability",
  "windows": ["widget"],
  "permissions": [
    "core:default",
    "core:window:allow-show",
    "core:window:allow-hide",
    "core:event:default"
  ]
}
```

**Adaptation Notes:**

- Create separate capability files for `widget` and `settings` if permissions diverge.
- Do not give the widget direct filesystem, shell, or network permissions.
- Keep Discord and Codex capture behind Tauri commands implemented in Rust.

### Pattern 8: Generated Rust-to-TypeScript Bindings

**Source:** [dannysmith/tauri-template](https://github.com/dannysmith/tauri-template) - `src-tauri/src/types.rs`, `src-tauri/src/lib.rs`
**Applies To:** ARCH.md Schema/Type Generation, ProviderSnapshot, AppState, AppConfig, AppError.

**Code Reference:**

```rust
// Pattern sketch derived from tauri-template's Serde + Specta model setup.
// Attribution: https://github.com/dannysmith/tauri-template
#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct ProviderSnapshot {
    pub schema_version: u16,
    pub provider_id: String,
    pub scraped_at: String,
    pub limits: Vec<LimitWindow>,
}
```

**Adaptation Notes:**

- Use `specta` during scaffolding and keep generated bindings aligned with ARCH.md source-of-truth Rust models.
- Generate bindings from `core/ida-core` models, not from React-local copies.
- Ensure frontend tests import the generated shape rather than hand-maintained duplicates.

## Anti-Patterns to Avoid

### Anti-Pattern 1: Broad App Architecture by Copying a Large Product

**Seen In:** `berbicanes/apiark`, `screenpipe/screenpipe`
**Issue:** Large Tauri products accumulate subsystems that Ida does not need in the MVP: SQLite databases, OAuth, plugin systems, proxying, multiple protocols, licensing, cloud-adjacent integrations, and complex workspace state.
**Our Approach Instead:** Follow ARCH.md Directory Structure: `apps/desktop/`, `core/ida-core/`, `providers/codex/`, `providers/claude/` placeholder. Keep persistence to JSON/NDJSON for MVP.

### Anti-Pattern 2: Direct File Writes for Critical State

**Seen In:** `Rench321/sklad` for some app data writes
**Issue:** Direct writes can corrupt `latest.json` or config during crashes or interrupted writes.
**Our Approach Instead:** Use temp-file-then-rename atomic writes per ARCH.md Persistence and Storage sections.

### Anti-Pattern 3: Webview-Owned Shell Execution

**Seen In:** Generic Tauri shell plugin examples
**Issue:** Letting React invoke shell commands would broaden the webview's capability surface and risk exposing Codex/WSL execution details.
**Our Approach Instead:** Keep Codex native/WSL capture in `providers/codex/` Rust code. Use argument arrays, typed capture errors, and Rust-side redaction per ARCH.md Security Considerations.

### Anti-Pattern 4: Secrets in General App State

**Seen In:** Some broad settings structs in evaluated apps
**Issue:** If secrets travel through generic frontend state or snapshots, they can leak into logs, generated bindings, tests, or screenshots.
**Our Approach Instead:** ARCH.md requires Discord webhook URLs to be local secrets and never appear in snapshots, history, alert state entries, generated bindings, logs, fixtures, or UI dumps.

### Anti-Pattern 5: Product Code in the Prototype Folder

**Seen In:** Not in external repos; identified from PRD risk.
**Issue:** Mixing new Tauri app code into `local/` would weaken the current Codex Limits Prototype as a known-good reference.
**Our Approach Instead:** Preserve `local/` as reference/prototype and build Ida under the Product Structure described in PRD/ARCH.

### Anti-Pattern 6: Dashboard-Scale UI for a Widget Product

**Seen In:** Large local-first desktop apps and broad templates
**Issue:** Heavy sidebars, command palettes, onboarding flows, or dashboard layouts would distract from Ida's "glanceable first" widget mission.
**Our Approach Instead:** Use only the compact-control patterns that support `WidgetWindow.tsx` and `SettingsWindow.tsx`; keep the widget at roughly 280px by 160px and verify no text overlaps.

## Dependency Discoveries

| Library | Purpose | Version | Consider Adding? |
|---------|---------|---------|------------------|
| `tauri-plugin-window-state` | Persist and restore window size/position. | 2.x | Maybe. Useful for widget preferences, but ARCH.md already models `WidgetPreferences`; review whether custom storage gives better control. |
| `tauri-plugin-single-instance` | Prevent duplicate app instances and focus/show existing windows. | 2.x | Yes. Useful for a tray utility and now reflected in ARCH.md. |
| `tauri-plugin-log` | Tauri-integrated logs to stdout, files, and optionally webview. | 2.x | Maybe. ARCH.md specifies `tracing`; use only if it complements Rust tracing without leaking secrets. |
| `async-trait` | Async provider trait support for `UsageProvider::refresh`. | 0.1.x | Yes. ARCH.md already shows `#[async_trait::async_trait]`; explicitly add to Rust dependencies. |
| `thiserror` | Typed Rust error enums with ergonomic display/source handling. | 2.x or current stable | Yes. Useful for `AppError`, provider capture errors, parser errors, and storage errors. |
| `specta` | Rust-to-TypeScript type and command binding generation. | 2.x/current | Already in ARCH.md as one option. Tauri-template is a useful implementation reference. |
| `tauri-plugin-process` | App process/restart helpers. | 2.x | Maybe. Useful later for updater/restart flows, not required for MVP widget. |
| `tauri-plugin-shell` | Shell command plugin. | 2.x | No for Codex capture. Keep provider execution in Rust; use only for tightly scoped non-secret frontend operations if later approved. |
| `tempfile` | Safer temp files in tests and atomic-write implementations. | 3.x | Maybe. Useful for storage tests and parser fixture tests. |

**Note:** Any additions require updating ARCH.md Tech Stack before Phase 3.

## Decisions After Review

1. Telegram is not in the MVP. Discord is the only external webhook alert channel for now, with Telegram reserved for a future channel implementation.

2. Tauri v2 is confirmed as the desktop framework because it keeps the Windows MVP lightweight while preserving a practical path to macOS and Linux later.

3. `tauri-plugin-single-instance` should be included so Ida does not create duplicate tray icons or polling loops.

4. `specta` is the chosen Rust-to-TypeScript binding generator for scaffolding.

5. `thiserror` and `async-trait` should be included in the Rust dependencies because they directly support the approved error strategy and provider trait.

6. Ida's `WidgetPreferences` remains the source of truth for widget position, size, visibility, and always-on-top state. `tauri-plugin-window-state` remains optional after smoke testing if it improves cross-platform window restore behavior.

## Open Questions for Review

1. No strong open-source example was found for parsing Codex usage output specifically. The Codex provider/parser likely needs a custom implementation using local prototype behavior plus dedicated fixtures.

## Validation Checklist

- [x] All recommended repos have compatible licenses: MIT or Apache-2.0.
- [x] All recommended repos were active within the last 2 years.
- [x] Patterns extracted align with the ARCH.md stack: Tauri v2, Rust, React/TypeScript, Tokio, serde, reqwest, local files.
- [x] Code references include attribution to source repositories and are pattern sketches, not large copied blocks.
- [x] Anti-patterns are documented to prevent future implementation mistakes.
- [x] No recommended repo contradicts PRD scope; oversized repos are explicitly scoped to narrow patterns only.
- [x] Adaptation notes reference ARCH.md Dictionary terms such as Provider Snapshot, Limit Window, Widget, Tray Icon, AppState, AppConfig, Structured Error, and Alert Deduplication.
- [x] Open questions are actionable and ready for stakeholder/architecture review.
