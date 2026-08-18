# Ida Usage Monitor Brainstorming

Date: 2026-05-03

## Context

The current repo is a Codex usage monitor prototype. It scrapes local Codex CLI usage, writes JSON snapshots, renders a browser dashboard, keeps short history, and can send alerts through Discord or Telegram.

The new product idea from `notes.txt` is broader:

- Extend Ida to monitor both Codex and Claude.
- Start with desktop apps/widgets for Windows, macOS, and Linux.
- Later add phone apps/widgets for Android and iOS.

This is currently a brainstorming and research phase. No implementation decision should be treated as final yet.

## High-Level Product Direction

The best direction is to turn the current prototype into an "Ida Core" usage-monitoring layer, then build multiple app surfaces on top of it.

Suggested architecture:

```text
Codex provider + Claude provider
        |
        v
Ida Core collector
        |
        v
Local store: SQLite plus latest JSON snapshot
        |
        v
Desktop tray, floating widget, dashboard, alerts, optional sync
        |
        v
Mobile apps and phone widgets
```

The important product distinction:

- Desktop machines can run local CLI collectors.
- Phones generally cannot run `codex` or `claude` directly, so phone apps/widgets should read synced snapshots produced by a desktop collector.

## Current Repo Observations

Current useful pieces:

- `local/monitor.sh` already captures Codex status through a PTY-backed interactive Codex session.
- It parses 5-hour and weekly usage, reset times, account info, scrape time, sample interval, and history window.
- It writes `local/data.json` and `local/history.json`.
- It supports threshold alerts through Discord and Telegram.
- It optionally syncs the current snapshot/history to GitHub Gist.
- `local/dashboard.html` proves the basic display surface: two main usage cards plus a history chart.

Current cleanup needs before scaling:

- `local/monitor.sh` still contains older duplicate functions (`scrape_status` and `write_local`) that no longer appear to be on the main path.
- `local/dashboard.html` calls `refresh()` twice on page load.
- README/dashboard text has visible encoding corruption in several places.
- Current schema is Codex-specific and should be generalized before adding Claude.
- History should eventually move from flat JSON to SQLite or a small embedded database layer, while still emitting JSON snapshots for widgets and static dashboards.

## Provider Strategy

### Codex

Codex currently appears to require scraping the terminal status output. The existing PTY approach is a workable prototype.

Recommended next steps:

- Wrap Codex status capture behind a `CodexProvider` interface.
- Keep parser fixtures with saved raw Codex status output.
- Add tests for parse success, missing fields, changed formatting, and stale output.
- Prefer a future machine-readable Codex command if OpenAI adds one.
- Normalize "percent left" into a shared provider schema.

### Claude

Claude may be easier than Codex if Ida uses Claude Code's status-line data rather than terminal scraping.

Research note from current docs: Claude Code status line JSON can include rate limit fields such as 5-hour and 7-day usage percentages/reset times after API activity, for Pro/Max-style usage contexts.

Recommended next steps:

- Build a `ClaudeProvider` that consumes Claude status-line JSON.
- Avoid terminal scraping as the first approach.
- Provide a small bridge script/config for users that writes Claude usage snapshots where Ida Core can read them.
- Normalize "percent used" into the same shared schema as Codex.

## Shared Data Model

Use both provider-specific fields and normalized fields.

Possible normalized shape:

```json
{
  "schema_version": 1,
  "scraped_at": "2026-05-03T00:00:00Z",
  "sources": [
    {
      "provider": "codex",
      "account_label": "optional",
      "status": "ok",
      "limits": [
        {
          "id": "codex_5h",
          "label": "5-hour",
          "window": "5h",
          "used_pct": 11,
          "remaining_pct": 89,
          "resets_at": "optional ISO time or provider text",
          "raw_reset_text": "18:21"
        },
        {
          "id": "codex_weekly",
          "label": "weekly",
          "window": "7d",
          "used_pct": 32,
          "remaining_pct": 68,
          "resets_at": "optional ISO time or provider text",
          "raw_reset_text": "22:06 on 18 Mar"
        }
      ]
    }
  ]
}
```

Important schema detail:

- Codex reports remaining percentage.
- Claude status-line data appears to report used percentage.
- Store both `used_pct` and `remaining_pct` after normalization.

## Desktop App Direction

Recommended starting framework: Tauri.

Why:

- Good fit for a lightweight cross-platform desktop app.
- Supports Rust backend logic and a web UI frontend.
- Can ship a tray/menu-bar app.
- Can run local commands or sidecars on desktop.
- Gives some path toward mobile reuse, though desktop and mobile capabilities differ.

Desktop MVP surfaces:

- Tray/menu-bar indicator.
- Small floating always-on-top widget window.
- Full dashboard window.
- Settings for providers, polling interval, alert thresholds, sync destination, privacy controls.
- Native notifications.

Desktop platform priorities:

1. macOS and Windows tray/menu-bar style app.
2. Linux AppIndicator/tray support where available.
3. Native widget integrations only after the core is proven.

Native widget thoughts:

- Windows Widgets require a packaged provider model and are more involved than a tray app.
- macOS widgets require WidgetKit and shared app-group storage.
- Linux does not have one universal widget system, so tray/AppIndicator is the realistic first target.

## Mobile App Direction

Mobile should follow desktop, not lead.

Reason:

- Mobile apps cannot reliably run the local authenticated desktop CLIs.
- Mobile needs a synced data source from the desktop collector.

Possible sync options:

1. GitHub Gist, continuing the current prototype path.
2. Ida Cloud, for a polished product experience.
3. User-owned endpoint, such as self-hosted storage, Tailscale, or Cloudflare Tunnel.

Mobile surfaces:

- iOS app with WidgetKit widgets.
- Android app with Jetpack Glance widgets.
- Push notification support for low-limit alerts.
- Read-only usage display by default.

Privacy posture:

- Phone apps should not need OpenAI or Anthropic credentials.
- Sync should store only sanitized usage snapshots by default.
- Account labels should be optional and redacted by default.

## PRD Recommendation

Yes, this should get a PRD before major scaffolding.

Reason:

- The project is shifting from a small Codex dashboard to a cross-platform product.
- The provider model, data model, sync model, and privacy posture need to be settled early.
- A PRD will help avoid turning the current prototype structure into the long-term architecture by accident.

Suggested branch:

```text
codex/ida-prd-and-scaffold
```

Suggested docs:

```text
docs/PRD.md
docs/architecture.md
docs/provider-contract.md
docs/mobile-strategy.md
```

Suggested repo shape after PRD:

```text
apps/
  desktop/
core/
  ida-core/
providers/
  codex/
  claude/
docs/
local/
```

Keep `local/` as the prototype/reference implementation while new scaffolding is created.

## Proposed Phases

### Phase 0: PRD and Architecture

- Write the PRD.
- Define target users and jobs-to-be-done.
- Define provider contract.
- Define local data schema.
- Define privacy and sync posture.
- Decide desktop MVP scope.

### Phase 1: Core Extraction

- Extract parser/provider logic out of `local/monitor.sh`.
- Add Codex parser fixtures and tests.
- Add normalized snapshot writer.
- Keep compatibility with existing `data.json` and `history.json` where practical.

### Phase 2: Claude Provider

- Implement Claude status-line ingestion.
- Normalize Claude rate-limit fields.
- Add tests and fixtures.
- Show Codex and Claude together in a unified dashboard.

### Phase 3: Desktop App MVP

- Scaffold Tauri desktop app.
- Add tray/menu-bar UI.
- Add small floating widget window.
- Add settings.
- Add native notifications.
- Package for at least one OS first, then expand.

### Phase 4: Sync

- Keep GitHub Gist as the prototype sync target.
- Add a cleaner sync abstraction.
- Define sanitized snapshot rules.
- Prepare for Ida Cloud or user-owned endpoint later.

### Phase 5: Mobile Apps and Widgets

- Build read-only mobile apps that consume synced snapshots.
- Add iOS WidgetKit widget.
- Add Android Glance widget.
- Add push or scheduled refresh behavior.

## Open Questions

- Is Ida intended to be a personal tool, a public open-source project, or a commercial product?
- Should desktop data stay fully local by default?
- Is GitHub Gist acceptable as a prototype sync layer, or should we skip directly to a proper backend?
- Which desktop OS should be the first real packaged target?
- Should the first UI be functional/plain, or should it preserve the current Codex Limits visual identity?
- How much should the app expose account/model names, given privacy concerns?
- Should alerts be local notifications first, Discord/Telegram first, or both?

## Source Notes From Research

Useful docs checked during brainstorming:

- Claude Code status line: https://code.claude.com/docs/en/statusline
- Claude Code costs/usage: https://code.claude.com/docs/en/costs
- OpenAI Codex plan usage: https://help.openai.com/en/articles/11369540-codex-in-chatgpt
- Tauri sidecars: https://v2.tauri.app/develop/sidecar/
- Tauri mobile development: https://v2.tauri.app/develop/
- Windows Widgets: https://learn.microsoft.com/en-us/windows/apps/develop/widgets/widget-providers
- Apple WidgetKit: https://developer.apple.com/documentation/WidgetKit/
- Android Jetpack Glance: https://developer.android.com/develop/ui/compose/glance

