# ARCH: Ida Desktop Usage Widget MVP

## 1. Overview

Ida is a local-first desktop utility that monitors local AI coding usage limits and keeps the current risk visible in a small floating widget and tray app. The MVP is a Tauri v2 desktop application with a React/TypeScript web UI, a Rust native shell, Rust core/provider crates, and file-based local persistence. The architecture is a single-process modular desktop monolith: UI, polling, provider capture, parsing, storage, settings, and alert orchestration ship together, while provider boundaries remain clean enough to add Claude later.

The first implementation target is Windows, but platform services must be designed behind traits so macOS and Linux desktop builds can follow without replacing the core model. Codex collection should attempt native Codex CLI capture first and fall back to WSL on Windows when native capture is unavailable or unreliable.

## 2. Dictionary (Ubiquitous Language)

| Term | Definition | Example |
|------|------------|---------|
| Ida | The new desktop product described by the PRD. | Ida runs in the tray and shows Codex remaining usage. |
| Codex Limits Prototype | The existing working prototype in `local/`. It is reference material, not the new product app. | `local/monitor.sh` keeps working while Ida is built elsewhere. |
| Prototype Folder | The repository folder `local/`, including `monitor.sh`, `dashboard.html`, `data.json`, and `history.json`. | Agents may read `local/monitor.sh` but must not add new desktop app code there. |
| Product Structure | The new non-colliding folders for Ida application, core logic, and providers. | `apps/desktop/`, `core/ida-core/`, and `providers/codex/`. |
| Desktop App | The packaged Tauri app that owns windows, tray behavior, settings UI, notifications, and lifecycle. | `apps/desktop/` contains the Tauri app. |
| Core | Provider-neutral Rust logic for schemas, storage, polling, stale detection, alert orchestration, and shared validation. | `core/ida-core/` validates snapshots before storage. |
| Provider | A module that captures and normalizes usage data from one external local tool or service. | Codex is the MVP provider; Claude is future. |
| Provider Boundary | The interface between provider-specific capture/parsing and provider-neutral Ida state. | The widget consumes `ProviderSnapshot`, not raw Codex output. |
| Codex Provider | The MVP provider that reads local Codex usage and converts it into normalized snapshots. | `providers/codex/` parses 5-hour and weekly limits. |
| Claude Provider | A future provider placeholder. No Claude implementation is in MVP scope. | `providers/claude/README.md` can document future shape. |
| Provider Contract | The trait and schema a provider must implement to produce normalized usage data. | `UsageProvider::refresh()` returns a `ProviderReadResult`. |
| Provider Snapshot | A successful normalized usage reading from one provider at one scrape time. | Codex snapshot with two limit windows. |
| Latest Snapshot | The most recent successful provider snapshot stored on disk. Failed scrapes do not replace it. | `latest.json` remains the 10:00 AM success after a 10:15 AM failure. |
| History | Short local append-only usage history for stale detection, debugging, and future charting. | `history.ndjson` keeps recent snapshots. |
| Limit Window | One quota window reported by a provider. | Codex has `5h` and `weekly` windows. |
| 5-hour Window | Codex rolling short-term usage window. | `5h` has 42 percent remaining. |
| Weekly Window | Codex longer usage window. | `weekly` resets Monday at 9:00 AM. |
| Remaining Percentage | Percentage of a limit still available, normalized to `0..100`. | `remaining_pct: 62`. |
| Used Percentage | Percentage of a limit already consumed, normalized to `0..100`. | `used_pct: 38` when remaining is 62. |
| Reset Time | Provider-reported or parsed time when a limit window resets. | `resets_at: 2026-05-03T23:20:00Z`. |
| Raw Reset Text | Provider reset text kept when an exact timestamp cannot be parsed. | `raw_reset_text: "22:06 on 18 Mar"`. |
| Scraped At | UTC timestamp for when usage data was captured. | `scraped_at: 2026-05-03T17:15:00Z`. |
| Provider Status | Overall provider state for the current app view. Valid values are `ok`, `stale`, `partial`, `unavailable`, and `error`. | Codex is `unavailable` when the CLI is not installed. |
| Limit Status | UI risk band for a limit. Valid values are `healthy`, `watch`, `low`, `critical`, `stale`, and `error`. | A limit at 8 percent remaining is `critical`. |
| Healthy | Limit status for 50 percent or more remaining. | Weekly at 71 percent is healthy. |
| Watch | Limit status for 25 to 49 percent remaining. | 5-hour at 42 percent is watch. |
| Low | Limit status for 10 to 24 percent remaining. | 5-hour at 18 percent is low. |
| Critical | Limit status below 10 percent remaining. | 5-hour at 4 percent is critical. |
| Stale | A view state where latest successful values exist but are older than the freshness threshold or the current scrape failed. | Widget shows 10:00 AM values as stale after a 10:15 AM failure. |
| Error State | A state where no usable current or previous values can be shown. | First launch shows "Codex not authenticated." |
| Structured Error | A typed error with code, message, details, and operation ID. | `CODEX_NOT_FOUND` tells the UI what action to show. |
| Collector | Provider code that executes local capture commands and obtains raw output. | The Codex collector launches `codex` or `wsl codex`. |
| Capture | The process of invoking Codex and collecting status output. | Native capture runs `codex /status` or an interactive PTY flow. |
| Parser | Provider code that converts raw output into normalized data. | Parser extracts `5h limit` and `% left`. |
| Raw Output | Untrusted provider-specific command output before normalization. | ANSI terminal output from Codex. |
| Provider Metadata | Provider-specific details allowed only under `metadata`. | Codex account label and raw parser version. |
| Widget | The primary small UI surface that displays limit status. | The widget floats near VS Code. |
| Floating Widget | A movable always-on-top desktop window. | User drags Ida to the corner of the screen. |
| Limit Row | One visual row/card inside the widget for a limit window. | The 5-hour row displays percentage, status, and reset. |
| Status Color | Color cue for limit or provider status. Color is never the only signal. | Amber plus "Watch". |
| Status Label | Text label matching the current status. | "Critical" appears beside a red indicator. |
| Tray Icon | System tray/menu bar indicator owned by the desktop app. | Clicking the tray icon opens controls. |
| Tray Menu | Native menu attached to the tray icon. | Show Widget, Hide Widget, Refresh Now, Settings, Quit. |
| Show Widget | Tray command that makes the floating widget visible. | User restores Ida after hiding it. |
| Hide Widget | Tray command that hides the widget while keeping the app running. | Ida remains in the tray. |
| Refresh Now | Tray or UI command that triggers an immediate provider scrape. | User clicks Refresh Now after signing into Codex. |
| Settings | Full in-app settings window for polling, freshness, alerts, Discord webhook, and widget preferences. | User changes polling interval to 15 minutes. |
| Quit | Command that closes the widget, stops polling, removes tray icon, and exits the process. | User selects Quit from the tray menu. |
| Polling | Background recurring provider refresh. | Ida polls every 900 seconds by default. |
| Polling Interval | Configurable seconds between scheduled refreshes. | Default is 900 seconds. |
| Freshness Threshold | Configurable age after which latest data is marked stale. | Default is 1800 seconds, two missed default polls. |
| Alert | Notification produced when remaining usage crosses a threshold. | Discord message for 5-hour below 25 percent. |
| Alert Threshold | Configured remaining percentage that triggers an alert when crossed downward. | Defaults are 75, 50, 25, 10, and 5. |
| Threshold Crossing | Event where remaining usage moves from above a threshold to at or below it. | 51 percent to 49 percent crosses 50. |
| Alert Deduplication | Suppression of repeat alerts for the same provider, limit, threshold, channel, and reset window. | No second 25 percent Discord alert until the 5-hour window resets. |
| Reset Window Key | Stable deduplication key derived from `resets_at` or `raw_reset_text`. | `codex:5h:2026-05-03T23:20:00Z`. |
| Native Notification | OS notification sent through Tauri notification APIs. | Windows toast for critical usage. |
| Discord Webhook | MVP external alert channel. The webhook URL is a local secret. | Ida posts a JSON payload to Discord. |
| Telegram Alert | Future alert channel from the PRD, deferred behind the alert channel interface. | Telegram can be added after Discord proves the path. |
| Configuration | Local user settings validated by core before use. | Polling, stale threshold, alert thresholds, Discord webhook. |
| Config File | Local JSON file storing validated configuration outside the repository. | `%APPDATA%/Ida/config.json` on Windows. |
| Secret | Sensitive value that must never be stored in snapshots or logs. | Discord webhook URL. |
| Snapshot Store | File-based persistence for latest snapshot and history. | `latest.json` and `history.ndjson`. |
| Alert State | Local deduplication state for alert delivery. | `alert-state.json` records the last sent threshold. |
| Widget Preferences | Persisted widget size, position, visibility, and always-on-top preference. | Widget opens where the user left it. |
| User | The local human using Ida. There is no Ida account entity in the MVP. | One Windows profile runs one Ida instance. |
| Account | Provider account label reported by Codex. It is not an Ida login. | `account` may be present in Codex metadata. |
| Developer | Contributor building or extending Ida. | Developer runs tests and creates providers. |
| Technical User | User comfortable building Ida from source. | Technical user follows open-source build docs. |
| Convenience-First Prosumer | User who wants a prebuilt app with minimal setup. | Future paid convenience build user. |
| Build Output | Generated binaries, installers, bundles, and compiled frontend assets. | Tauri `target/` output is not committed. |
| Open-Source Build Path | Documentation that lets technical users run or package Ida from source. | README shows install, dev, and build commands. |
| Paid Convenience Build | Future prebuilt distribution path. It has no license enforcement in the MVP. | Signed installer may be paid later. |
| WSL | Windows Subsystem for Linux, used as a fallback Codex capture environment on Windows. | Windows app invokes `wsl codex` if native capture fails. |
| Native Codex Capture | Direct capture from the current OS without WSL. | Windows native `codex` first, then WSL fallback. |
| Codex CLI | Local OpenAI Codex command-line tool already authenticated by the user. | `codex` is installed and signed in. |
| Typecheck | Static verification for Rust and TypeScript types. | `cargo check` and `pnpm typecheck`. |
| Lint | Static style and correctness checks. | `cargo clippy` and `pnpm lint`. |
| Minimum Widget Size | Smallest supported widget layout target. | About 280px wide by 160px tall. |
| Always-on-top | Window behavior where the widget stays above normal windows. | Widget remains visible while VS Code is focused. |

## 3. Tech Stack

| Layer | Technology | Version | Notes |
|-------|------------|---------|-------|
| Desktop Shell | Tauri | 2.x | Cross-platform desktop runtime for Windows first, with macOS and Linux readiness. |
| Native Runtime | Rust | 1.82+ stable | Pin with `rust-toolchain.toml`; use stable only. |
| Native Async Runtime | Tokio | 1.x | Background polling, capture timeouts, alert delivery. |
| Core Serialization | serde, serde_json | 1.x | All persisted schemas and Tauri command payloads. |
| Schema/Type Generation | specta | 2.x or current stable | Generate TypeScript bindings from Rust source-of-truth models. Chosen to reduce scaffold ambiguity and match researched Tauri patterns. |
| Error Modeling | thiserror | 2.x or current stable | Typed Rust error enums for provider, parser, storage, alert, and command errors before conversion to `AppError`. |
| Async Trait Helpers | async-trait | 0.1.x | Supports the async `UsageProvider` trait used by provider implementations. |
| Logging | tracing, tracing-subscriber | 0.1.x, 0.3.x | Structured local logs with secret redaction. |
| HTTP Client | reqwest | 0.12.x | Discord webhook delivery from Rust only. |
| App Directories | Tauri path APIs plus directories crate if needed | Tauri 2.x, directories 5.x | Store config/state in OS app directories, never in repo root. |
| Frontend Runtime | Node.js | 24 LTS preferred, 22.12+ minimum | Required for current Vite toolchain; do not use Node 20 as the baseline after April 30, 2026. |
| Package Manager | pnpm | 10.x | Deterministic frontend installs via `pnpm-lock.yaml`. |
| Frontend Build | Vite | 7.x | React TypeScript template; packaged by Tauri. |
| Frontend UI | React | 19.x | Widget, mini panel, and full settings window. |
| Frontend Language | TypeScript | 5.8+ | Strict mode; generated Rust bindings are imported by UI code. |
| Styling | Plain CSS modules or scoped CSS with CSS custom properties | N/A | Keep dependency weight low; no dashboard framework. |
| Icons | lucide-react | 0.5xx+ | Use existing icons for UI buttons and settings controls. |
| Native Notifications | Tauri notification plugin | 2.x | Windows notifications in MVP; macOS/Linux support later. |
| Tray/Menu | Tauri tray/menu APIs | 2.x | Show/Hide Widget, Refresh Now, Settings, Quit. |
| Single Instance | tauri-plugin-single-instance | 2.x | Prevent duplicate tray icons/windows; second launch focuses or shows the existing Ida instance. |
| Widget Window State | Ida `WidgetPreferences`; optional Tauri window-state plugin after smoke testing | schema_version 1, plugin 2.x if adopted | Keep Ida's preferences schema as source of truth; only add plugin support if it improves cross-platform window restore behavior. |
| Persistence | JSON and NDJSON files | schema_version 1 | No database for MVP. Atomic writes for JSON; append/trim for NDJSON. |
| Desktop Tests | cargo test, cargo nextest optional | Current stable | Unit and integration tests for core/provider crates. |
| Frontend Tests | Vitest, React Testing Library | Current stable | Component logic and settings validation UI tests. |
| Visual/E2E Tests | Playwright | Current stable | Browser verification for widget/settings surfaces; Tauri smoke tests when practical. |
| Formatting/Linting | rustfmt, clippy, ESLint, Prettier | Current stable | Required before task completion. |
| Packaging | Tauri bundler | 2.x | Windows installer first; macOS/Linux packaging wired but may be release-gated. |

Version notes: Tauri docs require Rust and current Node LTS for development, and current Vite requires Node 20.19+ or 22.12+. Since Node 20 reached end of life on April 30, 2026, Ida should baseline Node 24 LTS where available and accept Node 22.12+ for contributors still on maintenance LTS.

## 4. Data Models

All models are source-of-truth Rust structs in `core/ida-core/` unless explicitly provider-specific. TypeScript models are generated from Rust and committed only if the chosen generator expects generated bindings in source control.

#### Entity: ProviderSnapshot

Represents one successful normalized provider reading. Failed scrapes never overwrite the latest successful snapshot.

| Field | Type | Constraints | Description |
|-------|------|-------------|-------------|
| schema_version | integer | required, equals `1` | Snapshot schema version. |
| provider_id | string | required, lowercase slug | Provider key, initially `codex`. |
| provider_name | string | required | Display name, initially `Codex`. |
| provider_status | enum | `ok`, `partial` | Successful snapshots are `ok` unless one expected limit is missing. |
| scraped_at | datetime | required, UTC ISO 8601 | Capture timestamp. |
| capture_method | enum | `native`, `wsl`, `fixture`, `unknown` | How the data was captured. |
| source_platform | enum | `windows`, `macos`, `linux`, `unknown` | OS where capture ran. |
| limits | array of LimitWindow | required, at least 1 | Normalized limit windows. Codex MVP expects exactly `5h` and `weekly` when fully available. |
| metadata | ProviderMetadata | required | Provider-specific metadata. |

**Relationships:**
- ProviderSnapshot has many LimitWindow records.
- ProviderSnapshot has one ProviderMetadata object.
- Latest snapshot store contains zero or one ProviderSnapshot per provider.
- History contains many ProviderSnapshot records.

#### Entity: LimitWindow

Represents one usage limit window in a provider-neutral shape.

| Field | Type | Constraints | Description |
|-------|------|-------------|-------------|
| id | string | required, unique within provider snapshot | Stable key such as `5h` or `weekly`. |
| label | string | required, max 40 chars | User-facing label such as `5-hour` or `Weekly`. |
| window | string | required | Provider-neutral window identifier. |
| remaining_pct | integer | required, `0..100` | Percentage remaining. |
| used_pct | integer | required, `0..100`, `remaining_pct + used_pct = 100` when known | Percentage used. |
| resets_at | datetime or null | UTC ISO 8601 when parseable | Parsed reset timestamp. |
| raw_reset_text | string or null | max 120 chars | Provider reset text when exact timestamp is unavailable. |
| status | enum | `healthy`, `watch`, `low`, `critical`, `stale`, `error` | Risk band for display. |
| status_reason | string or null | max 160 chars | Short reason for unusual status. |
| metadata | object | provider-specific, non-secret | Limit-level provider details. |

**Relationships:**
- LimitWindow belongs to one ProviderSnapshot.
- AlertStateEntry references LimitWindow by `provider_id` and `limit_id`.

#### Entity: ProviderMetadata

Stores non-secret provider-specific details.

| Field | Type | Constraints | Description |
|-------|------|-------------|-------------|
| account_label | string or null | max 160 chars, non-secret | Codex account text if reported. |
| raw_model_label | string or null | max 80 chars | Model label if reported by Codex. |
| parser_version | string | required | Parser version used to produce the snapshot. |
| raw_fields | object | optional, non-secret only | Codex-specific raw parsed fields. |

**Relationships:**
- ProviderMetadata belongs to one ProviderSnapshot.
- ProviderMetadata must not contain credentials, access tokens, webhook URLs, or full raw terminal transcripts.

#### Entity: ProviderReadResult

The result of a provider refresh attempt.

| Field | Type | Constraints | Description |
|-------|------|-------------|-------------|
| attempted_at | datetime | required, UTC | When refresh started. |
| completed_at | datetime or null | UTC | When refresh completed. |
| provider_id | string | required | Provider key. |
| result_type | enum | `success`, `partial`, `failure` | Outcome category. |
| snapshot | ProviderSnapshot or null | present for `success` and sometimes `partial` | Normalized successful data. |
| error | AppError or null | present for `failure` and sometimes `partial` | Structured error. |

**Relationships:**
- Polling creates ProviderReadResult objects.
- Successful ProviderReadResult updates Latest Snapshot and History.
- Failed ProviderReadResult updates AppState error but not Latest Snapshot.

#### Entity: AppState

Current state used by the widget and tray.

| Field | Type | Constraints | Description |
|-------|------|-------------|-------------|
| schema_version | integer | required, equals `1` | App state schema version. |
| provider_id | string | required | Active provider, initially `codex`. |
| latest_snapshot | ProviderSnapshot or null | nullable | Most recent successful snapshot. |
| current_error | AppError or null | nullable | Most recent failed refresh error. |
| freshness_status | enum | `fresh`, `stale`, `unavailable`, `error` | UI freshness state. |
| last_attempted_at | datetime or null | UTC | Last refresh attempt start. |
| last_success_at | datetime or null | UTC | Last successful scrape time. |
| next_poll_at | datetime or null | UTC | Next scheduled poll. |
| effective_limits | array of LimitWindow | derived | Limits with stale/error status applied for UI. |

**Relationships:**
- AppState reads from Latest Snapshot, AppConfig, and latest ProviderReadResult.
- Tray visual state is derived from AppState.

#### Entity: AppConfig

Validated local configuration managed by the settings window.

| Field | Type | Constraints | Description |
|-------|------|-------------|-------------|
| schema_version | integer | required, equals `1` | Config schema version. |
| active_provider_id | string | required, default `codex` | Active provider. |
| polling_interval_seconds | integer | required, `60..86400`, default `900` | Scheduled refresh interval. |
| stale_after_seconds | integer | required, `120..172800`, default `1800` | Age after which latest data is stale. |
| alert_thresholds | array of integer | required, unique, descending, each `0..100`, default `[75, 50, 25, 10, 5]` | Remaining percentages that trigger alerts. |
| native_notifications_enabled | boolean | required, default `true` | Enables OS notifications. |
| discord_alerts_enabled | boolean | required, default `false` | Enables Discord webhook alert delivery. |
| discord_webhook_url | string or null | optional secret, valid Discord webhook URL when present | Local-only Discord webhook destination. |
| capture_mode | enum | `native_then_wsl`, `native_only`, `wsl_only` | Default `native_then_wsl`. |
| history_retention_hours | integer | required, `1..168`, default `24` | Retention cap for local history. |
| log_level | enum | `error`, `warn`, `info`, `debug` | Default `info`; debug still redacts secrets. |

**Relationships:**
- AppConfig controls Polling, Capture, Alerting, and History retention.
- Settings UI reads and updates AppConfig.
- AppConfig must never be committed to the repository.

#### Entity: WidgetPreferences

Persisted desktop window preferences.

| Field | Type | Constraints | Description |
|-------|------|-------------|-------------|
| schema_version | integer | required, equals `1` | Preferences schema version. |
| visible_on_launch | boolean | required, default `true` | Whether widget starts visible. |
| always_on_top | boolean | required, default `true` | Whether widget stays above normal windows. |
| position_x | integer or null | nullable | Last screen x coordinate. |
| position_y | integer or null | nullable | Last screen y coordinate. |
| width | integer | `280..800`, default `280` | Last widget width. |
| height | integer | `160..600`, default `160` | Last widget height. |
| display_id | string or null | nullable | Best-effort display identifier. |

**Relationships:**
- Desktop App applies WidgetPreferences during window creation.
- Widget UI and tray commands can update WidgetPreferences.

#### Entity: AlertState

Local deduplication state for alert delivery.

| Field | Type | Constraints | Description |
|-------|------|-------------|-------------|
| schema_version | integer | required, equals `1` | Alert state schema version. |
| entries | array of AlertStateEntry | required | Sent alert records retained for active reset windows. |
| updated_at | datetime | required, UTC | Last state write. |

**Relationships:**
- AlertState has many AlertStateEntry records.
- AlertState is pruned when reset windows change or entries exceed retention.

#### Entity: AlertStateEntry

One deduplication record.

| Field | Type | Constraints | Description |
|-------|------|-------------|-------------|
| provider_id | string | required | Provider key. |
| limit_id | string | required | Limit window key. |
| threshold | integer | required, `0..100` | Alert threshold crossed. |
| channel | enum | `native`, `discord` | MVP channels. |
| reset_window_key | string | required | `resets_at` when available, else stable raw reset text hash. |
| sent_at | datetime | required, UTC | Delivery attempt time. |
| delivery_status | enum | `sent`, `failed` | Last delivery outcome. |
| error_code | string or null | nullable | Error code for failed delivery. |

**Relationships:**
- AlertStateEntry references ProviderSnapshot through provider and reset window values, not by database foreign key.

#### Entity: AppError

Structured error returned to UI and stored as current failure context.

| Field | Type | Constraints | Description |
|-------|------|-------------|-------------|
| code | enum | required | Stable machine-readable error code. |
| message | string | required, max 240 chars | User-actionable message. |
| details | object | optional, redacted | Additional non-secret context. |
| operation_id | string | required UUID or ULID | Correlates UI errors and logs. |
| occurred_at | datetime | required, UTC | Error timestamp. |
| retryable | boolean | required | Whether user can retry. |

**Relationships:**
- ProviderReadResult may contain AppError.
- AppState may expose the latest AppError.

## 5. API Contracts

The MVP does not expose an HTTP API. The app surface is a Tauri command interface between the React UI/tray handlers and Rust native code. All commands return JSON-serializable Rust models and use the shared AppError shape on failure.

### Command: `get_app_state`

Return the current widget/tray state.

**Request Body:**

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| include_config_summary | boolean | No | If true, include non-secret effective config values. |

**Response:**

| Field | Type | Description |
|-------|------|-------------|
| state | AppState | Current provider snapshot/error/freshness state. |
| lowest_status | string | Most severe effective limit status for tray rendering. |
| config_summary | object or null | Non-secret settings summary when requested. |

**Errors:**
- `SNAPSHOT_CORRUPT`: latest snapshot file exists but cannot be parsed.
- `CONFIG_INVALID`: config file exists but fails validation.
- `FILE_IO_ERROR`: local state cannot be read.
- `INTERNAL_ERROR`: unexpected failure.

### Command: `refresh_usage`

Trigger an immediate provider refresh and update app state.

**Request Body:**

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| provider_id | string | No | Defaults to active provider, initially `codex`. |
| reason | string | No | `manual`, `startup`, `poll`, or `test`. |

**Response:**

| Field | Type | Description |
|-------|------|-------------|
| result | ProviderReadResult | Capture result. |
| state | AppState | Updated app state after applying result. |

**Errors:**
- `PROVIDER_NOT_FOUND`: requested provider is not registered.
- `CODEX_NOT_FOUND`: Codex CLI is not installed or not on PATH.
- `CODEX_UNAUTHENTICATED`: Codex appears installed but not signed in.
- `WSL_NOT_FOUND`: Windows fallback requested but WSL is unavailable.
- `WSL_UNAVAILABLE`: WSL exists but command execution failed.
- `CAPTURE_TIMEOUT`: provider capture exceeded timeout.
- `PARSER_FAILED`: raw output could not be parsed into a usable snapshot.
- `FILE_IO_ERROR`: successful snapshot could not be persisted.
- `INTERNAL_ERROR`: unexpected failure.

### Command: `get_config`

Return validated configuration for the settings UI. Secrets must be redacted.

**Request Body:**

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| include_secret_presence | boolean | No | If true, report whether secrets are set without returning values. |

**Response:**

| Field | Type | Description |
|-------|------|-------------|
| config | AppConfigRedacted | Config with `discord_webhook_url` removed or masked. |
| secret_presence | object | Example: `{ "discord_webhook_url": true }`. |
| config_path | string | Local path for troubleshooting, safe to show. |

**Errors:**
- `CONFIG_INVALID`: config file fails validation.
- `CONFIG_READ_FAILED`: config file cannot be read.
- `INTERNAL_ERROR`: unexpected failure.

### Command: `update_config`

Validate and persist settings from the settings window.

**Request Body:**

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| patch | AppConfigPatch | Yes | Partial config update. |
| secret_updates | object | No | Explicit secret writes, such as `discord_webhook_url`. Empty string clears a secret. |

**Response:**

| Field | Type | Description |
|-------|------|-------------|
| config | AppConfigRedacted | Updated redacted config. |
| restart_required | boolean | True only for settings that cannot apply live. |

**Errors:**
- `VALIDATION_ERROR`: patch fails schema validation.
- `DISCORD_WEBHOOK_INVALID`: webhook URL is malformed or not a Discord webhook URL.
- `CONFIG_WRITE_FAILED`: config cannot be written atomically.
- `INTERNAL_ERROR`: unexpected failure.

### Command: `get_widget_preferences`

Return persisted widget preferences.

**Request Body:** none.

**Response:**

| Field | Type | Description |
|-------|------|-------------|
| preferences | WidgetPreferences | Current persisted widget preferences. |

**Errors:**
- `PREFERENCES_INVALID`: preferences file fails validation and defaults were used.
- `FILE_IO_ERROR`: preferences cannot be read.
- `INTERNAL_ERROR`: unexpected failure.

### Command: `update_widget_preferences`

Persist widget position, size, visibility, and always-on-top preferences.

**Request Body:**

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| patch | WidgetPreferencesPatch | Yes | Partial preference update. |

**Response:**

| Field | Type | Description |
|-------|------|-------------|
| preferences | WidgetPreferences | Updated preferences. |

**Errors:**
- `VALIDATION_ERROR`: size or coordinate values are invalid.
- `PREFERENCES_WRITE_FAILED`: preferences cannot be written atomically.
- `INTERNAL_ERROR`: unexpected failure.

### Command: `test_discord_webhook`

Send a test Discord alert using the configured or supplied webhook.

**Request Body:**

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| webhook_url | string | No | Optional unsaved webhook URL to test. If omitted, use configured secret. |

**Response:**

| Field | Type | Description |
|-------|------|-------------|
| delivery_status | string | `sent` or `failed`. |
| status_code | integer or null | HTTP status from Discord when available. |
| message | string | Short user-facing result. |

**Errors:**
- `DISCORD_NOT_CONFIGURED`: no webhook URL is available.
- `DISCORD_WEBHOOK_INVALID`: webhook URL is malformed.
- `DISCORD_DELIVERY_FAILED`: Discord returned failure or network failed.
- `INTERNAL_ERROR`: unexpected failure.

### Command: `open_config_directory`

Open the OS-specific Ida configuration directory.

**Request Body:** none.

**Response:**

| Field | Type | Description |
|-------|------|-------------|
| opened | boolean | True if the OS accepted the open request. |
| path | string | Directory path. |

**Errors:**
- `FILE_IO_ERROR`: config directory cannot be created or opened.
- `INTERNAL_ERROR`: unexpected failure.

### Command: `quit_app`

Stop polling, persist current preferences, close windows, remove tray icon, and exit.

**Request Body:** none.

**Response:**

| Field | Type | Description |
|-------|------|-------------|
| accepted | boolean | True once shutdown has started. |

**Errors:**
- `INTERNAL_ERROR`: unexpected failure while beginning shutdown.

### Rust Provider Trait

All providers implement this provider-neutral contract.

```rust
#[async_trait::async_trait]
pub trait UsageProvider: Send + Sync {
    fn id(&self) -> &'static str;
    fn display_name(&self) -> &'static str;
    async fn refresh(&self, config: &ProviderRuntimeConfig) -> ProviderReadResult;
}
```

Codex-specific capture and parsing stays in `providers/codex/`. The widget and settings UI must not import Codex parser types directly.

## 6. Directory Structure

| Path | Purpose | Contains |
|------|---------|----------|
| `apps/` | Product app entry points | Desktop app now; future app shells if needed. |
| `apps/desktop/` | Tauri desktop application | Web UI, Tauri config, native shell integration. |
| `apps/desktop/src/` | React/TypeScript UI | Widget, mini panel, settings window, generated bindings imports. |
| `apps/desktop/src/components/` | Shared UI components | Limit rows, status indicators, settings controls. |
| `apps/desktop/src/windows/` | Window-level React views | `WidgetWindow.tsx`, `SettingsWindow.tsx`. |
| `apps/desktop/src/bindings/` | Generated TypeScript bindings | Generated from Rust models. |
| `apps/desktop/src-tauri/` | Tauri app crate | Window/tray setup, command registration, app lifecycle. |
| `apps/desktop/src-tauri/icons/` | Native app icons | Status icon assets for tray/build. |
| `core/` | Shared product logic workspace | Provider-neutral crates. |
| `core/ida-core/` | Rust source of truth for Ida domain | Models, config, storage, polling, alerts, errors, type generation. |
| `core/ida-core/src/models/` | Domain models | Snapshot, config, app state, alert state, widget preferences. |
| `core/ida-core/src/storage/` | File persistence | Atomic JSON writes, NDJSON history, corruption handling. |
| `core/ida-core/src/alerts/` | Alert orchestration | Threshold crossing, dedupe, native/Discord channel interface. |
| `core/ida-core/src/platform/` | Platform abstractions | App dirs, notifications, process execution traits. |
| `providers/` | Provider implementations | Codex now, Claude placeholder later. |
| `providers/codex/` | Codex provider crate | Native/WSL capture, PTY fallback, parser, fixtures, provider adapter. |
| `providers/codex/fixtures/` | Parser fixtures | Success, missing fields, unauthenticated, changed format. |
| `providers/claude/` | Future provider placeholder | README or stub only; no MVP implementation. |
| `tests/` | Cross-crate integration tests | End-to-end core/provider tests when not colocated in crates. |
| `tests/fixtures/` | Shared test fixtures | Sanitized provider outputs and config examples. |
| `docs/` | Project documentation | `PRD.md`, `ARCH.md`, build/run docs. |
| `directives/` | Agent task instructions | Implementation task files such as `001_initial_setup.md`. |
| `execution/` | Deterministic automation scripts | Setup checks, release helpers, fixture update scripts. |
| `local/` | Existing prototype/reference implementation | `monitor.sh`, dashboard, prototype data. Do not place new Ida app code here. |
| `.tmp/` | Agent scratchpad | Temporary files, gitignored. |
| `.github/` | CI workflows when added | Typecheck, lint, tests, builds. |

Storage is outside the repository:

| Logical File | Windows Path | macOS Path | Linux Path | Purpose |
|--------------|--------------|------------|------------|---------|
| Config | `%APPDATA%/Ida/config.json` | `~/Library/Application Support/Ida/config.json` | `$XDG_CONFIG_HOME/ida/config.json` | User settings and Discord secret. |
| Latest Snapshot | `%LOCALAPPDATA%/Ida/state/latest.json` | `~/Library/Application Support/Ida/state/latest.json` | `$XDG_STATE_HOME/ida/latest.json` | Latest successful snapshot. |
| History | `%LOCALAPPDATA%/Ida/state/history.ndjson` | `~/Library/Application Support/Ida/state/history.ndjson` | `$XDG_STATE_HOME/ida/history.ndjson` | Short local history. |
| Alert State | `%LOCALAPPDATA%/Ida/state/alert-state.json` | `~/Library/Application Support/Ida/state/alert-state.json` | `$XDG_STATE_HOME/ida/alert-state.json` | Alert deduplication state. |
| Widget Preferences | `%APPDATA%/Ida/widget-preferences.json` | `~/Library/Application Support/Ida/widget-preferences.json` | `$XDG_CONFIG_HOME/ida/widget-preferences.json` | Window position, size, visibility, always-on-top. |
| Logs | `%LOCALAPPDATA%/Ida/logs/ida.log` | `~/Library/Logs/Ida/ida.log` | `$XDG_STATE_HOME/ida/logs/ida.log` | Local logs with secrets redacted. |

## 7. Error Handling Strategy

### Tauri Command Errors

All command errors return this JSON shape:

```json
{
  "error": {
    "code": "CODEX_NOT_FOUND",
    "message": "Codex CLI was not found. Install Codex or update PATH, then refresh.",
    "details": {},
    "operation_id": "01HYEXAMPLE000000000000000",
    "occurred_at": "2026-05-03T17:15:00Z",
    "retryable": true
  }
}
```

### Error Codes

| Code | Category | When Used |
|------|----------|-----------|
| `VALIDATION_ERROR` | User input | Settings, preferences, or command payload fails schema validation. |
| `CONFIG_INVALID` | Config | Existing config file is readable but invalid. |
| `CONFIG_READ_FAILED` | Config | Config file cannot be read. |
| `CONFIG_WRITE_FAILED` | Config | Config cannot be written atomically. |
| `PREFERENCES_INVALID` | Preferences | Widget preferences fail validation and defaults are used. |
| `PREFERENCES_WRITE_FAILED` | Preferences | Widget preferences cannot be persisted. |
| `SNAPSHOT_NOT_FOUND` | Storage | No latest snapshot exists yet. |
| `SNAPSHOT_CORRUPT` | Storage | Latest snapshot file exists but cannot be decoded or validated. |
| `HISTORY_WRITE_FAILED` | Storage | History append or retention trim failed. |
| `ALERT_STATE_WRITE_FAILED` | Storage | Alert dedupe state cannot be persisted. |
| `FILE_IO_ERROR` | Storage | Generic local file read/write/open failure. |
| `PROVIDER_NOT_FOUND` | Provider | Unknown provider requested. |
| `CODEX_NOT_FOUND` | Codex | Codex executable is not found in native PATH. |
| `CODEX_UNAUTHENTICATED` | Codex | Codex appears installed but status output indicates sign-in is required. |
| `WSL_NOT_FOUND` | WSL | Windows fallback requested but `wsl.exe` is missing. |
| `WSL_UNAVAILABLE` | WSL | WSL command cannot run or selected distro is unavailable. |
| `CAPTURE_TIMEOUT` | Provider | Provider capture exceeded configured timeout. |
| `CAPTURE_FAILED` | Provider | Provider command exited unsuccessfully for a known reason. |
| `PARSER_FAILED` | Provider | Raw output cannot be parsed into required normalized fields. |
| `PARTIAL_SNAPSHOT` | Provider | At least one expected limit parsed but another is missing. |
| `DISCORD_NOT_CONFIGURED` | Alert | Discord delivery requested without a configured webhook. |
| `DISCORD_WEBHOOK_INVALID` | Alert | Discord webhook URL is malformed or unsupported. |
| `DISCORD_DELIVERY_FAILED` | Alert | Discord request failed or returned non-2xx status. |
| `NOTIFICATIONS_UNAVAILABLE` | Alert | OS notification API is unavailable or denied. |
| `INTERNAL_ERROR` | Internal | Unexpected failure. |

### Logging

- Use `tracing` for structured logs.
- Every command invocation and provider refresh gets an `operation_id`.
- Log full internal error chains locally, but redact secrets before serialization.
- Never log Discord webhook URLs, future provider tokens, OpenAI credentials, Anthropic credentials, or raw terminal output that may contain sensitive account details.
- In UI copy, prefer short actionable messages over stack traces.

### Failure Behavior

- Failed scrapes do not overwrite `latest.json`.
- If `latest.json` exists and is valid, the widget shows last known values with stale status.
- If no usable snapshot exists, the widget shows an empty/error state, not placeholder percentages.
- A partial Codex parse may store a partial snapshot only when at least one limit window is valid and the missing limit is represented clearly as an error/partial state.
- Misconfigured alerts never block widget rendering or provider polling.

## 8. Security Considerations

### Authentication

- Ida has no app account, no backend session, and no hosted authentication in the MVP.
- The local OS user profile is the trust boundary.
- Codex authentication is owned by the Codex CLI. Ida only invokes local Codex status capture and must not handle OpenAI credentials.
- Token expiration and refresh are not Ida concerns in the MVP.

### Secrets Management

- MVP secret support is Discord webhook only.
- Store the Discord webhook URL in the local app config file, outside the repository.
- Never store Discord webhook URLs in snapshots, history, alert state entries, generated bindings, test fixtures, logs, screenshots, or committed docs.
- Settings UI must display secret presence or a masked value, never the full saved webhook after initial entry.
- Environment variable override `IDA_DISCORD_WEBHOOK_URL` may be supported for development, but UI-saved config remains the default user path.
- Future OS credential manager support may replace config-file secrets after MVP if needed.

### Input Validation

- Validate all Tauri command payloads through Rust models before applying changes.
- Validate percentages as integers in `0..100`.
- Validate polling and stale thresholds within configured bounds.
- Validate Discord webhook URLs as HTTPS Discord webhook endpoints.
- Reject invalid config values with `VALIDATION_ERROR` or a more specific code.
- No SQL is used in the MVP. No raw string interpolation into shell commands is allowed.
- Provider command execution must use argument arrays, not string-built shell commands.
- File uploads are not supported.

### CORS and WebView Policy

- No public HTTP API exists in the MVP, so CORS is not applicable.
- The React UI must call Rust through Tauri commands, not direct network calls for secrets.
- The webview Content Security Policy should default to local assets and Tauri IPC only.
- Discord webhook delivery must happen from Rust, not from the webview, to avoid exposing secrets.

### Local File Permissions

- Create app config/state directories with OS-default user-only permissions where available.
- Use atomic writes for JSON files to avoid corrupting config/snapshot state on crash.
- Treat corrupt files as recoverable errors with clear UI messages.

## 9. Integration Points

| System | Purpose | Auth Method | Rate Limits |
|--------|---------|-------------|-------------|
| Codex CLI | Local usage status capture. | User's existing local Codex authentication. Ida does not access credentials. | Local command; no Ida-owned rate limit. Poll conservatively, default 15 minutes. |
| WSL | Windows fallback execution path for Codex capture. | Local Windows user permissions. | Local subsystem; capture timeout applies. |
| Discord Webhook | MVP external alert delivery. | Webhook URL secret stored locally. | Discord applies webhook limits. Ida must send only threshold-crossing alerts and dedupe per reset window. |
| Native OS Notification Service | Local desktop notifications. | Local app permission and OS notification APIs. | OS-dependent. Failures do not block widget updates. |
| Local Filesystem | Config, latest snapshot, history, preferences, logs, alert state. | Local OS user permissions. | N/A. |

Telegram is not in the MVP architecture after stakeholder approval to start with Discord only. Keep the alert channel interface generic so Telegram can be added later without changing threshold or dedupe logic.

## 10. Non-Functional Requirements

| Requirement | Target | Measurement |
|-------------|--------|-------------|
| Local-first operation | No hosted backend, cloud account, sync, license server, or Ida account required. | App can run with only local Codex prerequisites and optional Discord webhook. |
| Supported users | One local user per OS profile. | No multi-user/team data model in MVP. |
| Startup freshness | Show live values within 60 seconds when Codex is installed and authenticated. | Time from app launch to successful first `ProviderSnapshot`. |
| Single instance behavior | A second Ida launch must not create a duplicate tray icon or polling loop. | Existing instance is focused or shown during desktop smoke testing. |
| UI state command latency | Less than 50ms p95 for cached state/config reads. | Tauri command timing excluding disk cold starts. |
| Manual refresh latency | Less than 30 seconds p95 for normal Codex capture. | Time from Refresh Now to ProviderReadResult on a healthy local setup. |
| Polling interval | Default 900 seconds, configurable from 60 seconds to 24 hours. | AppConfig validation. |
| Stale threshold | Default 1800 seconds, configurable. | AppState `freshness_status` changes when latest snapshot age exceeds threshold. |
| Widget size | Minimum about 280px by 160px without overlapping text. | Visual verification in browser and desktop smoke test. |
| Idle resource use | Less than 1 percent CPU while idle; memory target below 150MB for packaged app. | Local profiling on Windows during MVP validation. |
| Data retention | Latest snapshot retained until replaced; history retained 24 hours by default. | `history_retention_hours` and file trim tests. |
| Reliability | Scrape, alert, or config failures must not crash the app process. | Error injection tests and manual smoke tests. |
| Secret safety | No credentials or webhook secrets in snapshots, history, logs, fixtures, or UI state dumps. | Unit tests for redaction plus manual review. |
| Provider extensibility | Adding Claude must not require widget data-shape changes. | Widget uses ProviderSnapshot and LimitWindow only. |
| Cross-platform readiness | Platform-specific operations are behind traits or Tauri APIs. | Compile-time module boundaries and targeted platform smoke tests. |
| Build reproducibility | Rust and frontend dependencies are pinned. | `Cargo.lock`, `pnpm-lock.yaml`, `rust-toolchain.toml`. |

## 11. Open Technical Questions

| Question | Impact | Current Direction |
|----------|--------|-------------------|
| Exact minimum OS versions for public builds | Determines installer/support matrix. | Windows first; keep macOS/Linux ready in Tauri config but release-gate if untested. |
| Final Codex status capture command | Parser stability and capture complexity. | Try native status capture first; retain PTY fallback behavior based on prototype if direct command is unreliable. |
| Whether Codex exposes stable JSON status | Could simplify parsing and reduce breakage. | Detect and prefer JSON if available; otherwise parse terminal output behind Codex provider. |
| Open-source license with future paid builds | Affects commercial distribution clarity. | Repository currently has MIT license; confirm before public Ida release. |
| Code signing and auto-update path | Required for polished paid convenience builds. | Out of MVP unless packaging task explicitly adds it. |
| Full settings UI scope for first release | Determines implementation effort. | Include full in-app settings for MVP-critical config only: polling, stale threshold, alert thresholds, Discord, widget preferences. |
| History retention beyond 24 hours | Affects storage and future charting. | Keep NDJSON short and configurable; no SQLite until product needs exceed simple files. |
| Cross-platform notification edge cases | macOS/Linux permissions differ from Windows. | Windows validation first; abstract notification channel for later platform hardening. |

## 12. Validation Checklist

- [x] Every domain term from PRD User Stories is defined in Dictionary or intentionally clarified as out of MVP scope.
- [x] All User Stories have a clear implementation path through Tauri command, core service, provider module, storage model, or UI window.
- [x] Data models support all MVP functional requirements, including provider-neutral snapshots, latest storage, stale/error state, widget preferences, config, Discord alerts, and dedupe.
- [x] API contracts cover all user-facing operations: state read, refresh, settings read/write, widget preferences, Discord test, config directory, and quit.
- [x] Directory structure maps to Tauri, Rust core crates, provider crates, docs, directives, execution scripts, prototype reference, and scratch space.
- [x] Dependencies are version-constrained where stability matters.
- [x] Security section addresses local trust boundary, Codex-owned authentication, Discord secret handling, validation, and no public API.
- [x] Error codes exist for failure scenarios in User Stories, including missing Codex, unauthenticated Codex, missing WSL, parser failure, stale data, corrupt files, and alert delivery failure.
