# PRD: Ida Windows Usage Widget MVP

## Executive Summary

Ida is a local-first desktop usage monitor that helps AI coding users avoid unexpected interruptions from Codex usage limits. The MVP focuses on a polished Windows floating widget and tray indicator that show Codex 5-hour and weekly remaining usage, reset times, and status colors at a glance. The product is designed for the agentic coding era, where long-running coding workflows can fail or stall if a user unknowingly reaches subscription limits. The first version ships Codex monitoring only, but its data model and provider boundary must be designed so Claude can be added later without rewriting the app.

## Mission and Core Principles

**Mission Statement:** Help AI coding users see usage-limit risk before it disrupts critical coding work.

**Core Principles:**
1. **Glanceable first** - The primary value must be visible in a tiny desktop widget without opening a dashboard.
2. **Local by default** - The MVP must run without cloud accounts, hosted services, or synced data.
3. **Trust the freshness** - Users must always know whether values are current, stale, or unavailable.
4. **Ship Codex, prepare for providers** - Codex works first, but the app must not hard-code the UI or storage to Codex-only assumptions.
5. **Open source with convenient paid builds** - Technical users can compile the project themselves, while non-technical users can later pay for a ready-to-run build.
6. **Protect the working prototype** - The existing MVP should remain usable as reference material while new Ida work happens in a separate structure.

## Target Users

**Primary: AI Coding Power Users**
- **Who they are:** Developers, builders, and prosumers using Codex heavily during daily coding work.
- **Technical comfort level:** Medium to high. They can install tools and follow setup instructions, but they still value a polished app that saves time.
- **Key needs:** See remaining limits quickly, avoid interrupted agentic sessions, and receive alerts before critical workflows are blocked.

**Secondary: Convenience-First Prosumers**
- **Who they are:** Users who rely on AI coding tools but do not want to compile software, configure scripts, or manage WSL details.
- **Technical comfort level:** Low to medium.
- **Key needs:** A prebuilt app that works with minimal setup and explains problems clearly when local prerequisites are missing.

**Persona Conflict**

When open-source flexibility conflicts with a simple packaged experience, the packaged experience should win for the default user flow. Technical users can still configure or build from source, but the main UX should assume the user wants a small, reliable desktop utility.

## Scope

### In Scope

- [ ] Windows-first MVP.
- [ ] New non-colliding repository structure for Ida app, core, and providers.
- [ ] Existing `local/` prototype preserved as a reference implementation during MVP development.
- [ ] WSL-backed Codex monitoring is acceptable for MVP if the desktop widget experience is polished.
- [ ] Codex provider that reads local Codex usage data and normalizes it into a shared provider schema.
- [ ] Floating always-on-top widget showing Codex 5-hour and weekly limits.
- [ ] Display of remaining percentage, reset time, and simple status color for each limit window.
- [ ] Tray icon with click-to-open mini panel or widget controls.
- [ ] Clear stale/error state when Codex status cannot be read.
- [ ] Local latest snapshot storage.
- [ ] Local short history storage if needed for stale detection and future charting.
- [ ] Native desktop notifications for low-remaining thresholds.
- [ ] Discord webhook alerts, reusing the current prototype behavior where practical.
- [ ] Minimal local configuration for polling interval, alert thresholds, and webhook destinations.
- [ ] Provider contract designed for future Claude support.
- [ ] Open-source build path documented for technical users.

### Out of Scope

- [ ] Claude provider implementation in the MVP.
- [ ] macOS or Linux packaged apps in the MVP.
- [ ] Mobile apps or phone widgets.
- [ ] Cloud sync, Ida Cloud, GitHub Gist sync, or cross-device data sync.
- [ ] Multi-user, team, or organization usage management.
- [ ] Telegram alert implementation in the MVP.
- [ ] In-app payments, license enforcement, user accounts, or subscription backend.
- [ ] Full analytics dashboard as the primary MVP experience.
- [ ] Native Windows Widgets platform integration.
- [ ] Automatic intervention when limits are low, such as stopping agents or changing models.
- [ ] Building the new desktop app directly inside the existing `local/` prototype folder.

## User Stories

### US-000: Create Non-Colliding Product Structure

**Description:** As a developer, I want the new Ida product code to live in a clean structure separate from the existing prototype so the working MVP does not break while the robust app is built.

**Example:** The current `local/monitor.sh`, `local/dashboard.html`, `local/data.json`, and `local/history.json` continue to work as the Codex Limits prototype. New Ida code is added under paths such as `apps/desktop/`, `core/ida-core/`, and `providers/codex/` instead of mixing desktop app code into `local/`.

**Acceptance Criteria:**
- [ ] Create or document a new top-level structure before adding desktop app implementation code.
- [ ] Keep `local/` as prototype/reference code unless a later task explicitly migrates or removes it.
- [ ] New app code lives under `apps/desktop/` or an equivalent dedicated app folder.
- [ ] Shared product logic lives under `core/` or an equivalent dedicated core folder.
- [ ] Provider-specific logic lives under `providers/` or an equivalent dedicated provider folder.
- [ ] No new desktop app build output is committed into the root or `local/` folder.
- [ ] README or docs explain which folders are prototype versus new product code.
- [ ] Typecheck/lint passes.

### US-001: Define Normalized Provider Snapshot

**Description:** As a developer, I want a provider-neutral snapshot format so Ida can support Codex now and Claude later.

**Example:** Codex reports "89% left" for the 5-hour window. Ida stores that as a normalized limit object with `remaining_pct: 89`, `used_pct: 11`, `window: "5h"`, and provider metadata identifying Codex.

**Acceptance Criteria:**
- [ ] Define a schema version field, such as `schema_version: 1`.
- [ ] Snapshot includes `scraped_at`, provider name, provider status, and an array of limit windows.
- [ ] Each limit includes `id`, `label`, `window`, `remaining_pct`, `used_pct`, `resets_at` or `raw_reset_text`, and status metadata.
- [ ] Codex-specific raw fields are allowed only inside provider-specific metadata.
- [ ] Schema supports future providers without changing the widget rendering contract.
- [ ] Typecheck/lint passes.

### US-002: Capture Codex Usage Locally

**Description:** As a user, I want Ida to read my local Codex usage so the widget reflects my real subscription limits.

**Example:** A user has Codex installed and authenticated in WSL. Ida runs the local collector, captures 5-hour and weekly usage, and writes a normalized snapshot showing 62% remaining for 5-hour usage and 78% remaining for weekly usage.

**Acceptance Criteria:**
- [ ] Collector can read Codex status from the local environment.
- [ ] Windows MVP may call into WSL if native Windows Codex status capture is not available.
- [ ] Parser extracts 5-hour remaining percentage, weekly remaining percentage, reset text, and scrape time.
- [ ] Parser handles missing fields by returning a structured error instead of crashing.
- [ ] A successful scrape writes a latest normalized snapshot.
- [ ] Typecheck/lint passes.

### US-003: Store Latest Snapshot Locally

**Description:** As a user, I want Ida to remember the latest successful usage data so the widget can remain useful during temporary scrape failures.

**Example:** Ida successfully reads Codex at 10:00 AM, then the next scrape fails at 10:15 AM. The widget continues showing the 10:00 AM values but marks them stale instead of pretending they are current.

**Acceptance Criteria:**
- [ ] Latest snapshot is stored locally on disk.
- [ ] Storage contains no OpenAI credentials, Anthropic credentials, API keys, or webhook secrets.
- [ ] Snapshot includes enough timestamp information to determine staleness.
- [ ] Failed scrapes do not overwrite the latest successful values with empty data.
- [ ] Corrupt local snapshot files are detected and reported as an error state.
- [ ] Typecheck/lint passes.

### US-004: Render Floating Usage Widget

**Description:** As an AI coding user, I want a small always-on-top widget so I can see Codex usage risk while working.

**Example:** The user is coding in VS Code with Ida floating in the corner. The widget shows "5-hour: 42% left, resets 6:20 PM" and "Weekly: 71% left, resets Monday 9:00 AM" with amber/green status colors.

**Acceptance Criteria:**
- [ ] Widget shows exactly two primary limit rows or cards: 5-hour and weekly.
- [ ] Each limit shows remaining percentage, reset time, and status color.
- [ ] Widget has a stale indicator when data is older than the configured freshness window.
- [ ] Widget is movable by the user.
- [ ] Widget can stay always on top.
- [ ] Widget does not require opening a browser dashboard.
- [ ] Text remains readable and non-overlapping at the minimum widget size.
- [ ] Typecheck/lint passes.
- [ ] Verify in browser using dev-browser skill.

### US-005: Provide Tray Indicator and Controls

**Description:** As a user, I want a tray icon so Ida stays accessible without occupying taskbar space.

**Example:** Ida runs in the background. The tray icon indicates the lowest remaining limit status. The user clicks the icon and can show/hide the widget, trigger refresh, and quit Ida.

**Acceptance Criteria:**
- [ ] App creates a Windows tray icon while running.
- [ ] Tray menu includes Show/Hide Widget, Refresh Now, Settings or Configure, and Quit.
- [ ] Tray visual state reflects the most severe current limit status where technically feasible.
- [ ] Refresh Now triggers a new Codex scrape and updates the widget after completion.
- [ ] Quit closes the widget and stops background polling.
- [ ] Typecheck/lint passes.
- [ ] Verify in browser using dev-browser skill.

### US-006: Alert on Low Remaining Usage

**Description:** As a user, I want alerts before I hit Codex limits so I can avoid starting work that may be interrupted.

**Example:** The 5-hour remaining limit drops below 25%. Ida sends a native Windows notification and posts to the configured Discord webhook once for that threshold.

**Acceptance Criteria:**
- [ ] User can configure alert thresholds as percentages remaining.
- [ ] Default thresholds are provided, such as 75, 50, 25, 10, and 5 percent remaining.
- [ ] Native Windows notifications fire when a threshold is crossed.
- [ ] Discord webhook alert fires when configured.
- [ ] Alerts are deduplicated so the same threshold does not spam repeatedly within the same reset window.
- [ ] Alerts include provider, limit window, remaining percentage, and reset time when available.
- [ ] Typecheck/lint passes.

### US-007: Handle Stale and Error States

**Description:** As a user, I want Ida to clearly explain when usage data is stale or unavailable so I do not trust bad information.

**Example:** Codex is not authenticated. Ida shows "Codex status unavailable" with the last successful values, timestamp, and a hint to open Codex and sign in.

**Acceptance Criteria:**
- [ ] Widget shows last known values when current scrape fails but previous data exists.
- [ ] Widget clearly labels stale data with the last successful scrape time.
- [ ] If no previous data exists, widget shows an empty/error state instead of placeholder percentages.
- [ ] Error details are short and user-actionable.
- [ ] App distinguishes between unauthenticated Codex, Codex not found, parser failure, and unknown failure where possible.
- [ ] Typecheck/lint passes.
- [ ] Verify in browser using dev-browser skill.

### US-008: Provide Minimal Configuration

**Description:** As a user, I want to configure polling and alerts without editing source code.

**Example:** The user sets polling to every 15 minutes, adds a Discord webhook, and changes the critical alert threshold to 10%. Ida saves those settings locally and uses them after restart.

**Acceptance Criteria:**
- [ ] Configuration supports polling interval.
- [ ] Configuration supports freshness/stale threshold.
- [ ] Configuration supports alert thresholds.
- [ ] Configuration supports Discord webhook settings.
- [ ] Secrets are stored locally and excluded from committed source files.
- [ ] Invalid configuration values are rejected with clear messages.
- [ ] Typecheck/lint passes.

### US-009: Document Open-Source Build Path

**Description:** As a technical user, I want clear build instructions so I can compile Ida myself instead of using a paid packaged build.

**Example:** A developer clones the repo, installs prerequisites, runs the documented build command, and gets a working Windows app that can monitor Codex through their local setup.

**Acceptance Criteria:**
- [ ] Documentation explains the MVP Windows setup path.
- [ ] Documentation lists required local dependencies, including Codex authentication and WSL if required.
- [ ] Documentation explains how to run from source.
- [ ] Documentation explains how to build a local app package if packaging exists.
- [ ] Documentation clarifies that paid convenience builds are separate from the open-source source code.
- [ ] Typecheck/lint passes.

### US-010: Preserve Prototype Compatibility Where Practical

**Description:** As a developer, I want to reuse the existing Codex monitor prototype where it helps so the MVP can move faster.

**Example:** The current `local/monitor.sh` parser already writes `data.json` and `history.json`. Ida may keep using those files temporarily or adapt the parser into a provider module while preserving a compatibility output for the old dashboard.

**Acceptance Criteria:**
- [ ] Existing useful parser behavior is either reused or covered by equivalent fixtures/tests.
- [ ] Existing `local/data.json` and `local/history.json` compatibility is preserved if it does not slow the MVP.
- [ ] Deprecated prototype paths are documented if they are no longer used.
- [ ] Any extraction from `local/` copies or ports behavior into the new structure rather than turning `local/` into the product app folder.
- [ ] No unrelated README/dashboard cleanup is required to complete the widget MVP.
- [ ] Typecheck/lint passes.

## Feature Specifications

### Repository Structure

**Route or location:** Repository root.

The current repository root contains the working Codex Limits MVP. The robust Ida product must use a new folder structure so app code, core logic, provider logic, docs, and prototype files do not collide. The existing `local/` folder should remain a prototype/reference implementation unless a future migration task explicitly changes that.

Recommended starting structure:

```text
apps/
  desktop/
    # Windows desktop widget/tray app
core/
  ida-core/
    # normalized schema, local store, polling, stale detection, alert orchestration
providers/
  codex/
    # Codex capture, parser, fixtures, provider adapter
  claude/
    # placeholder only for future provider contract; no MVP implementation
docs/
  PRD.md
local/
  # existing Codex Limits prototype/reference implementation
```

If the chosen desktop framework requires additional folders, the implementer may adapt this shape, but the separation rules remain the same: `local/` stays reference/prototype, app code stays under an app folder, shared logic stays under a core folder, and provider integrations stay under provider folders.

### Codex Provider and Snapshot Contract

**Route or location:** Core/provider layer. Exact path can be decided during implementation, but expected future structure is `core/`, `providers/codex/`, and `apps/desktop/`.

The Codex provider is responsible for capturing raw Codex status output, parsing it, and returning a normalized snapshot. The widget must consume only the normalized snapshot so Claude can later be added as another provider. Codex reports remaining percentage, while future providers may report used percentage, so the provider layer must normalize both `remaining_pct` and `used_pct`.

State is persisted locally as a latest snapshot and optionally a short history file or local database. The MVP should not store credentials in snapshots.

Edge cases:
- Codex command missing.
- Codex not authenticated.
- WSL missing or unavailable.
- Codex status output changes format.
- 5-hour value parses but weekly value is missing.
- Reset time is provider text instead of an ISO timestamp.

### Floating Widget

**Route or location:** Desktop app main widget window.

The widget is the primary product surface. It should be small enough to sit beside a coding editor and clear enough to read without interaction. It shows two limit windows: 5-hour and weekly. Each window includes a label, remaining percentage, reset time, and status indicator.

Suggested visual states:
- Healthy: 50% or more remaining.
- Watch: 25% to 49% remaining.
- Low: 10% to 24% remaining.
- Critical: below 10% remaining.
- Stale: latest successful data exists but is older than the freshness threshold.
- Error: no usable data exists.

The widget should be movable and support always-on-top behavior. If the widget is closed, Ida should keep running in the tray unless the user chooses Quit.

### Tray Indicator

**Route or location:** Windows system tray.

The tray icon keeps Ida accessible while minimizing screen clutter. The tray menu should let users show or hide the widget, refresh usage immediately, open configuration, and quit. If possible, the tray icon should reflect the most severe current state, such as healthy, watch, low, critical, stale, or error.

### Alerts

**Route or location:** Background polling/alert service.

Alerts fire when a limit crosses configured remaining-percentage thresholds. The MVP supports native Windows notifications plus Discord webhooks. Alerts should be deduplicated by provider, limit window, threshold, and reset window so users do not receive repeated notifications for the same threshold. Telegram is a future alert channel and should not be implemented in the MVP.

If an alert provider is misconfigured, Ida should show the failure in logs or settings without blocking the widget.

### Configuration

**Route or location:** Local config file and/or minimal settings UI.

The MVP needs configuration for polling interval, stale threshold, alert thresholds, and webhook settings. A full settings experience is not the core product, but users must not edit source code to configure Ida. If a settings UI is deferred, the config file must be documented clearly.

Secrets must be stored locally and excluded from Git. The MVP must not sync configuration or secrets to any external service.

### Distribution Model

**Route or location:** Repository documentation and future release pipeline.

Ida should be open source so technical users can inspect, modify, and compile it themselves. The commercial path is convenience-based: non-technical users can later subscribe or pay a small amount for a prebuilt, maintained package that saves setup time. The MVP PRD does not require payment processing, app accounts, license checks, or cloud subscription logic.

## Functional Requirements

- FR-1: The system must monitor Codex usage locally.
- FR-2: The system must support Windows as the first target platform.
- FR-3: The system may use WSL for Codex collection in the MVP if the user experience remains polished.
- FR-4: The system must parse Codex 5-hour remaining percentage.
- FR-5: The system must parse Codex weekly remaining percentage.
- FR-6: The system must capture reset time text for both Codex windows when available.
- FR-7: The system must normalize Codex data into a provider-neutral snapshot.
- FR-8: The system must store the latest successful snapshot locally.
- FR-9: The system must not store OpenAI credentials, Anthropic credentials, API keys, or webhook secrets in usage snapshots.
- FR-10: The system must render a floating desktop widget.
- FR-11: The widget must show 5-hour and weekly remaining usage.
- FR-12: The widget must show reset time for each visible limit when available.
- FR-13: The widget must show status colors or status labels for healthy, watch, low, critical, stale, and error states.
- FR-14: The widget must allow the user to move it.
- FR-15: The widget must support always-on-top behavior.
- FR-16: The system must provide a Windows tray icon while running.
- FR-17: The tray menu must allow the user to show/hide the widget.
- FR-18: The tray menu must allow the user to refresh usage manually.
- FR-19: The tray menu must allow the user to quit Ida.
- FR-20: The system must support native Windows notifications for threshold alerts.
- FR-21: The system must support Discord webhook alerts when configured.
- FR-22: The alert channel boundary must remain generic enough to add Telegram later without changing threshold or dedupe logic.
- FR-23: The system must deduplicate alerts for the same threshold within the same reset window.
- FR-24: The system must show last known values with a stale indicator when current scraping fails.
- FR-25: The system must show an actionable error state when no usable data exists.
- FR-26: The system must expose a minimal way to configure polling interval, stale threshold, alert thresholds, and webhook destinations.
- FR-27: The codebase must keep provider boundaries clear enough to add Claude later without rewriting the widget.
- FR-28: The repository must document how technical users can run or build the app from source.
- FR-29: The repository must place new Ida product code in a separate structure from the existing `local/` prototype.
- FR-30: The system must treat `local/` as reference/prototype code unless an explicit migration task says otherwise.
- FR-31: The repository must document the intended folder responsibilities for app, core, provider, docs, and prototype code.

## Non-Goals (Out of Scope Detail)

- **No Claude implementation in the MVP:** Claude is strategically important, but Codex must work first. The MVP should invest in provider boundaries so Claude support is additive later.
- **No cloud sync:** The first version should be fully local. Sync adds privacy, account, reliability, and backend questions that distract from proving the desktop widget.
- **No mobile apps or phone widgets:** Phones generally cannot run local authenticated coding CLIs directly. Mobile should wait until there is a reliable synced snapshot strategy.
- **No team management:** Shared limits, seats, admins, and organization reporting are different product problems. The MVP is for one user watching their own local usage.
- **No payment or license enforcement:** The open-source/commercial model is convenience-based for now. Building subscriptions into the app would require accounts or a backend, which conflicts with the local-only MVP.
- **No full dashboard-first rebuild:** A browser dashboard exists as a useful prototype, but the MVP value is a desktop widget and tray experience.
- **No native Windows Widgets integration:** Windows Widgets require a more involved packaged provider model. A floating widget and tray app are enough to validate demand.
- **No product work inside `local/`:** The current `local/` folder is a working MVP. Mixing new desktop app code into it would make the prototype harder to use as a known-good reference and increase migration risk.

## Design Considerations

- The widget should feel like a small system utility, not a marketing page or full dashboard.
- Suggested minimum widget size: approximately 280px wide by 160px tall.
- Text must remain readable at the minimum size and must not overlap.
- The two usage windows should be visually parallel so users can compare 5-hour and weekly status quickly.
- Color must not be the only signal. Include text labels such as Healthy, Watch, Low, Critical, Stale, or Error.
- Reset time should be visible but secondary to remaining percentage.
- The stale state must be obvious enough that users do not trust old numbers during critical work.
- The tray menu should use familiar labels: Show Widget, Hide Widget, Refresh Now, Settings, Quit.
- The app should avoid noisy onboarding. The first successful run should immediately show the widget with live values.
- Empty/error states should be short, specific, and actionable, such as "Codex not authenticated. Open Codex once and sign in."

## Technical Considerations

- The current prototype lives in `local/monitor.sh`, `local/data.json`, `local/history.json`, and `local/dashboard.html`.
- New Ida implementation work should begin in a separate top-level structure such as `apps/`, `core/`, and `providers/`.
- The `local/` folder should be read as reference/prototype code. Useful behavior can be ported into the new structure, but new desktop app code should not be built there.
- Existing Codex capture currently uses a PTY-backed interactive Codex session. This can be reused or replaced behind a provider interface.
- WSL is acceptable for the Windows MVP if native Codex access is not ready.
- The app should detect missing WSL, missing Codex, and unauthenticated Codex separately where feasible.
- Tauri v2 is the chosen desktop framework for the MVP because it supports a lightweight Windows-first app while keeping macOS and Linux paths practical for future releases.
- Local storage can start with JSON snapshots. SQLite can be introduced later if history, analytics, or multi-provider data grows.
- Polling should default to a conservative interval such as 15 minutes unless the user changes it.
- A snapshot should be considered stale after a configurable threshold, such as two missed polling intervals.
- Webhook secrets must be excluded from Git and should not appear in logs.
- The provider contract should include both `remaining_pct` and `used_pct` because providers may report different forms.

## Implementation Phases

### Phase 1: Product Contract and Codex Provider

**Phase goal:** Create the new Ida structure and convert the current Codex-specific prototype behavior into a reliable provider-backed data source.

**Deliverables:**
- [ ] New non-colliding folder structure for app, core, providers, and docs.
- [ ] Documentation that marks `local/` as prototype/reference code.
- [ ] Normalized snapshot schema.
- [ ] Codex provider/parser.
- [ ] Local latest snapshot writer.
- [ ] Structured scrape errors.
- [ ] Parser fixtures for successful, missing-field, stale, and changed-format cases.

**Validation criteria:**
- [ ] New Ida code is not mixed into `local/`.
- [ ] A local Codex status scrape produces a valid normalized snapshot.
- [ ] Parser tests pass for expected and failure cases.
- [ ] Snapshot contains no credentials.

### Phase 2: Windows Widget and Tray MVP

**Phase goal:** Build the smallest polished desktop surface that shows Codex 5-hour and weekly usage.

**Deliverables:**
- [ ] Floating always-on-top widget.
- [ ] Two visible limit windows: 5-hour and weekly.
- [ ] Reset time and status indicators.
- [ ] Tray icon and tray menu.
- [ ] Manual refresh.
- [ ] Stale and error states.

**Validation criteria:**
- [ ] User can see Codex status without opening a browser.
- [ ] Widget remains readable at minimum size.
- [ ] Tray controls work on Windows.
- [ ] UI is verified in browser using dev-browser skill where applicable.

### Phase 3: Alerts and Configuration

**Phase goal:** Warn users before they hit usage limits and let them configure the basics.

**Deliverables:**
- [ ] Native Windows notifications.
- [ ] Discord webhook alerts.
- [ ] Generic alert channel boundary for future channels such as Telegram.
- [ ] Alert threshold configuration.
- [ ] Polling/stale threshold configuration.
- [ ] Alert deduplication.

**Validation criteria:**
- [ ] Alerts fire once when a threshold is crossed.
- [ ] Misconfigured webhooks do not crash the app.
- [ ] Settings persist after app restart.

### Phase 4: Distribution Readiness

**Phase goal:** Make the open-source and future paid-build paths clear.

**Deliverables:**
- [ ] Source run instructions.
- [ ] Build instructions.
- [ ] Windows setup prerequisites.
- [ ] Notes explaining open-source source code versus future paid convenience builds.

**Validation criteria:**
- [ ] A technical user can follow the docs to run from source.
- [ ] Documentation clearly names what is local-only and what is not included.

## Risks and Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| Codex status output changes and breaks parsing | High | Keep parser fixtures, return structured parser errors, and isolate parsing inside the Codex provider. |
| WSL setup makes the Windows MVP feel too technical | High | Detect missing prerequisites clearly, keep the widget polished, and document the setup path step by step. |
| Users trust stale values during critical work | High | Always show stale labels with last successful scrape time and never silently display old values as fresh. |
| Alert channels spam users | Medium | Deduplicate alerts by provider, limit window, threshold, and reset window. |
| Open-source plus paid builds creates confusion | Medium | Explain that source builds are free and paid/subscription builds are for convenience, updates, and reduced setup work. |

## Future Considerations

- Claude provider using a provider-specific ingestion path.
- macOS menu-bar app.
- Linux tray/AppIndicator support.
- Telegram alert channel.
- Native Windows Widgets integration.
- iOS app and WidgetKit widget.
- Android app and Jetpack Glance widget.
- Optional GitHub Gist sync.
- Optional Ida Cloud sync for non-technical users.
- Multi-provider dashboard with history charts.
- Multi-account support.
- Team/org usage views.
- Signed installers and auto-update support.
- Paid packaged builds or low-cost subscription distribution.

## Success Metrics

- A user can see Codex 5-hour and weekly remaining usage from the desktop without opening a browser.
- Successful Codex data appears in the widget within 60 seconds of app start when prerequisites are already configured.
- Widget clearly distinguishes healthy, watch, low, critical, stale, and error states.
- Alerts fire once when a configured threshold is crossed.
- The app can run locally with no cloud account, backend, or sync setup.
- A technical user can run the open-source project from documented instructions.
- The provider contract can support Claude later without changing the widget UI data shape.

## Open Questions

- What exact Windows versions should the MVP support?
- Should Ida require WSL for the first public Windows MVP, or should native Windows Codex capture be required before release?
- What license should the open-source project use if paid convenience builds are planned?
- What is the exact pricing model for prebuilt convenience builds, such as one-time purchase versus low-cost subscription?
- Should the first public release include a settings UI, or is a documented local config file acceptable?
- Should the widget position and always-on-top preference persist across restarts in the MVP?
- What is the exact stale threshold: one missed poll, two missed polls, or a fixed time such as 30 minutes?
