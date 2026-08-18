// Generated shape from ida-core Rust models. Keep this file aligned with
// `cargo test -p ida-core` binding registry coverage when Rust models change.

export type ErrorCode =
  | "VALIDATION_ERROR"
  | "CONFIG_INVALID"
  | "CONFIG_READ_FAILED"
  | "CONFIG_WRITE_FAILED"
  | "PREFERENCES_INVALID"
  | "PREFERENCES_WRITE_FAILED"
  | "SNAPSHOT_NOT_FOUND"
  | "SNAPSHOT_CORRUPT"
  | "HISTORY_WRITE_FAILED"
  | "ALERT_STATE_WRITE_FAILED"
  | "FILE_IO_ERROR"
  | "PROVIDER_NOT_FOUND"
  | "CODEX_NOT_FOUND"
  | "CODEX_UNAUTHENTICATED"
  | "WSL_NOT_FOUND"
  | "WSL_UNAVAILABLE"
  | "CAPTURE_TIMEOUT"
  | "CAPTURE_FAILED"
  | "PARSER_FAILED"
  | "PARTIAL_SNAPSHOT"
  | "DISCORD_NOT_CONFIGURED"
  | "DISCORD_WEBHOOK_INVALID"
  | "DISCORD_DELIVERY_FAILED"
  | "NOTIFICATIONS_UNAVAILABLE"
  | "INTERNAL_ERROR";

export interface AppError {
  code: ErrorCode;
  message: string;
  details: Record<string, string>;
  operation_id: string;
  occurred_at: string;
  retryable: boolean;
}

export type LimitStatus = "healthy" | "watch" | "low" | "critical" | "stale" | "error";

export type FreshnessStatus = "fresh" | "stale" | "unavailable" | "error";
export type CaptureMode = "native_then_wsl" | "native_only" | "wsl_only";
export type LogLevel = "error" | "warn" | "info" | "debug";

export interface LimitWindow {
  id: string;
  label: string;
  window: string;
  remaining_pct: number;
  used_pct: number;
  resets_at: string | null;
  raw_reset_text: string | null;
  status: LimitStatus;
  status_reason: string | null;
  metadata: Record<string, string>;
}

export interface ProviderMetadata {
  account_label: string | null;
  raw_model_label: string | null;
  parser_version: string;
  raw_fields: Record<string, string>;
}

export interface ProviderSnapshot {
  schema_version: number;
  provider_id: string;
  provider_name: string;
  provider_status: "ok" | "partial";
  scraped_at: string;
  capture_method: "native" | "wsl" | "fixture" | "unknown";
  source_platform: "windows" | "macos" | "linux" | "unknown";
  limits: LimitWindow[];
  metadata: ProviderMetadata;
}

export interface ProviderReadResult {
  attempted_at: string;
  completed_at: string | null;
  provider_id: string;
  result_type: "success" | "partial" | "failure";
  snapshot: ProviderSnapshot | null;
  error: AppError | null;
}

export interface AppState {
  schema_version: number;
  provider_id: string;
  latest_snapshot: ProviderSnapshot | null;
  current_error: AppError | null;
  freshness_status: FreshnessStatus;
  last_attempted_at: string | null;
  last_success_at: string | null;
  next_poll_at: string | null;
  effective_limits: LimitWindow[];
}

export interface AppConfigRedacted {
  schema_version: number;
  active_provider_id: string;
  polling_interval_seconds: number;
  stale_after_seconds: number;
  alert_thresholds: number[];
  native_notifications_enabled: boolean;
  discord_alerts_enabled: boolean;
  discord_webhook_configured: boolean;
  discord_webhook_masked: string | null;
  capture_mode: CaptureMode;
  history_retention_hours: number;
  log_level: LogLevel;
  config_path: string | null;
}

export interface SecretPresence {
  discord_webhook_url: boolean;
}

export interface AppConfigPatch {
  active_provider_id?: string;
  polling_interval_seconds?: number;
  stale_after_seconds?: number;
  alert_thresholds?: number[];
  native_notifications_enabled?: boolean;
  discord_alerts_enabled?: boolean;
  capture_mode?: CaptureMode;
  history_retention_hours?: number;
  log_level?: LogLevel;
}

export interface SecretUpdates {
  discord_webhook_url?: string;
}

export interface WidgetPreferences {
  schema_version: number;
  visible_on_launch: boolean;
  always_on_top: boolean;
  position_x: number | null;
  position_y: number | null;
  width: number;
  height: number;
  display_id: string | null;
}

export interface WidgetPreferencesPatch {
  visible_on_launch?: boolean;
  always_on_top?: boolean;
  position_x?: number | null;
  position_y?: number | null;
  width?: number;
  height?: number;
  display_id?: string | null;
}

export interface GetAppStateResponse {
  state: AppState;
  lowest_status: LimitStatus;
  config_summary: AppConfigRedacted | null;
}

export interface RefreshOutcome {
  result: ProviderReadResult;
  state: AppState;
}

export interface GetConfigResponse {
  config: AppConfigRedacted;
  secret_presence: SecretPresence;
  config_path: string;
}

export interface UpdateConfigResponse {
  config: AppConfigRedacted;
  restart_required: boolean;
}

export interface GetWidgetPreferencesResponse {
  preferences: WidgetPreferences;
}

export interface UpdateWidgetPreferencesResponse {
  preferences: WidgetPreferences;
}

export interface OpenDirectoryResponse {
  opened: boolean;
  path: string;
}

export interface DiscordTestResult {
  delivery_status: string;
  status_code: number | null;
  message: string;
}
