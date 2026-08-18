import { invoke } from "@tauri-apps/api/core";
import type {
  AppConfigPatch,
  DiscordTestResult,
  GetAppStateResponse,
  GetConfigResponse,
  GetWidgetPreferencesResponse,
  OpenDirectoryResponse,
  RefreshOutcome,
  SecretUpdates,
  UpdateConfigResponse,
  UpdateWidgetPreferencesResponse,
  WidgetPreferencesPatch,
} from "./bindings/ida";
import { sampleAppState } from "./sampleState";

const fallbackConfig: GetConfigResponse = {
  config: {
    schema_version: 1,
    active_provider_id: "codex",
    polling_interval_seconds: 900,
    stale_after_seconds: 1800,
    alert_thresholds: [75, 50, 25, 10, 5],
    native_notifications_enabled: true,
    discord_alerts_enabled: false,
    discord_webhook_configured: false,
    discord_webhook_masked: null,
    capture_mode: "native_then_wsl",
    history_retention_hours: 24,
    log_level: "info",
    config_path: null,
  },
  secret_presence: { discord_webhook_url: false },
  config_path: "",
};

export async function getAppState(
  includeConfigSummary = false,
): Promise<GetAppStateResponse> {
  return invoke<GetAppStateResponse>("get_app_state", {
    include_config_summary: includeConfigSummary,
  }).catch(() => ({
    state: sampleAppState,
    lowest_status: "healthy",
    config_summary: includeConfigSummary ? fallbackConfig.config : null,
  }));
}

export async function refreshUsage(reason = "manual"): Promise<RefreshOutcome> {
  return invoke<RefreshOutcome>("refresh_usage", {
    request: { reason },
  }).catch(() => ({
    result: {
      attempted_at: new Date().toISOString(),
      completed_at: new Date().toISOString(),
      provider_id: "codex",
      result_type: "success",
      snapshot: sampleAppState.latest_snapshot,
      error: null,
    },
    state: sampleAppState,
  }));
}

export async function getConfig(): Promise<GetConfigResponse> {
  return invoke<GetConfigResponse>("get_config").catch(() => fallbackConfig);
}

export async function updateConfig(
  patch: AppConfigPatch,
  secretUpdates?: SecretUpdates,
): Promise<UpdateConfigResponse> {
  return invoke<UpdateConfigResponse>("update_config", {
    request: { patch, secret_updates: secretUpdates },
  });
}

export async function getWidgetPreferences(): Promise<GetWidgetPreferencesResponse> {
  return invoke<GetWidgetPreferencesResponse>("get_widget_preferences").catch(() => ({
    preferences: {
      schema_version: 1,
      visible_on_launch: true,
      always_on_top: true,
      position_x: null,
      position_y: null,
      width: 280,
      height: 160,
      display_id: null,
    },
  }));
}

export async function updateWidgetPreferences(
  patch: WidgetPreferencesPatch,
): Promise<UpdateWidgetPreferencesResponse> {
  return invoke<UpdateWidgetPreferencesResponse>("update_widget_preferences", {
    request: { patch },
  });
}

export async function openConfigDirectory(): Promise<OpenDirectoryResponse> {
  return invoke<OpenDirectoryResponse>("open_config_directory");
}

export async function testDiscordWebhook(
  webhookUrl?: string,
): Promise<DiscordTestResult> {
  return invoke<DiscordTestResult>("test_discord_webhook", {
    request: { webhook_url: webhookUrl || null },
  });
}
