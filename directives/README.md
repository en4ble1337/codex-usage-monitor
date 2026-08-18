# Ida Directive Sequence and Coverage

This index maps the implementation path from Directive 001 setup through the MVP directives. Directive 001 is referenced as the existing environment setup prerequisite even though it is not currently present in this repository checkout.

## Directive Sequence Overview

1. Directive 001: Initial Setup - prerequisite supplied by planning context.
2. Directive 002: Product Workspace and Scaffold - depends on 001.
3. Directive 003: Core Domain Models and Provider Contract - depends on 002.
4. Directive 004: Local Snapshot Storage and App State - depends on 003.
5. Directive 005: Configuration and Widget Preferences Core - depends on 004.
6. Directive 006: Codex Parser and Prototype Compatibility - depends on 003.
7. Directive 007: Codex Capture Provider - depends on 005 and 006.
8. Directive 008: Polling and Refresh Orchestration - depends on 004, 005, and 007.
9. Directive 009: Tauri Command Bridge - depends on 008.
10. Directive 010: Floating Widget Happy Path - depends on 009.
11. Directive 011: Stale and Error Widget States - depends on 010.
12. Directive 012: Tray Indicator and Controls - depends on 011.
13. Directive 013: Alert Thresholds and Native Notifications - depends on 008.
14. Directive 014: Discord Webhook Alerts - depends on 013.
15. Directive 015: Settings Window and Configuration UX - depends on 012 and 014.
16. Directive 016: Distribution Documentation and Release Smoke - depends on 015.

The chain is acyclic. Directive 002 depends only on Directive 001. Later directives list only earlier directives as prerequisites.

## PRD User Story Coverage

| User Story | Implementing Directives |
|---|---|
| US-000 Create Non-Colliding Product Structure | 002, 016 |
| US-001 Define Normalized Provider Snapshot | 003 |
| US-002 Capture Codex Usage Locally | 006, 007, 008, 009 |
| US-003 Store Latest Snapshot Locally | 004, 008 |
| US-004 Render Floating Usage Widget | 010, 011 |
| US-005 Provide Tray Indicator and Controls | 012 |
| US-006 Alert on Low Remaining Usage | 013, 014, 015 |
| US-007 Handle Stale and Error States | 004, 011 |
| US-008 Provide Minimal Configuration | 005, 015 |
| US-009 Document Open-Source Build Path | 016 |
| US-010 Preserve Prototype Compatibility Where Practical | 006, 016 |

## PRD Functional Requirement Coverage

| Functional Requirement | Implementing Directives |
|---|---|
| FR-1 Monitor Codex usage locally | 006, 007, 008, 009 |
| FR-2 Support Windows first | 002, 007, 009, 012, 016 |
| FR-3 May use WSL for collection | 007, 016 |
| FR-4 Parse Codex 5-hour remaining percentage | 006, 007 |
| FR-5 Parse Codex weekly remaining percentage | 006, 007 |
| FR-6 Capture reset time text | 006, 007 |
| FR-7 Normalize Codex data | 003, 006, 007 |
| FR-8 Store latest successful snapshot locally | 004, 008 |
| FR-9 Do not store credentials or webhook secrets in snapshots | 003, 004, 005, 014 |
| FR-10 Render floating desktop widget | 010 |
| FR-11 Show 5-hour and weekly usage | 010 |
| FR-12 Show reset time | 010 |
| FR-13 Show status colors or labels | 010, 011 |
| FR-14 Widget is movable | 010 |
| FR-15 Widget supports always-on-top | 010 |
| FR-16 Provide Windows tray icon | 012 |
| FR-17 Tray can show/hide widget | 012 |
| FR-18 Tray can refresh usage manually | 012 |
| FR-19 Tray can quit Ida | 012 |
| FR-20 Native Windows notifications | 013 |
| FR-21 Discord webhook alerts | 014, 015 |
| FR-22 Generic alert channel boundary | 013, 014 |
| FR-23 Alert deduplication | 013, 014 |
| FR-24 Show last known values as stale | 004, 008, 011 |
| FR-25 Show actionable empty/error state | 004, 011 |
| FR-26 Minimal configuration | 005, 009, 015 |
| FR-27 Provider boundaries for Claude later | 003, 006, 007 |
| FR-28 Document run/build from source | 016 |
| FR-29 Separate Ida code from local prototype | 002, 016 |
| FR-30 Treat local/ as reference/prototype | 002, 006, 016 |
| FR-31 Document folder responsibilities | 002, 016 |

## ARCH.md API Endpoint Coverage

| API Contract | Implementing Directive |
|---|---|
| `get_app_state` | 009 |
| `refresh_usage` | 009 |
| `get_config` | 009 |
| `update_config` | 009 |
| `get_widget_preferences` | 009 |
| `update_widget_preferences` | 009 |
| `test_discord_webhook` | 014 |
| `open_config_directory` | 009 |
| `quit_app` | 009 |

## ARCH.md Data Model Coverage

| Data Model Entity | Created In Directive |
|---|---|
| ProviderSnapshot | 003 |
| LimitWindow | 003 |
| ProviderMetadata | 003 |
| ProviderReadResult | 003 |
| AppState | 003 |
| AppConfig | 003 |
| WidgetPreferences | 003 |
| AlertState | 003 |
| AlertStateEntry | 003 |
| AppError | 003 |

## PRD Phase Coverage

| PRD Phase | Directives |
|---|---|
| Phase 1: Product Contract and Codex Provider | 002, 003, 004, 005, 006, 007, 008 |
| Phase 2: Windows Widget and Tray MVP | 009, 010, 011, 012 |
| Phase 3: Alerts and Configuration | 013, 014, 015 |
| Phase 4: Distribution Readiness | 016 |
