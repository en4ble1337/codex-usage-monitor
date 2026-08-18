import { FolderOpen, Send, Save } from "lucide-react";
import { FormEvent, useEffect, useState } from "react";
import type {
  AppConfigRedacted,
  CaptureMode,
  WidgetPreferences,
} from "../bindings/ida";
import {
  getConfig,
  getWidgetPreferences,
  openConfigDirectory,
  testDiscordWebhook,
  updateConfig,
  updateWidgetPreferences,
} from "../commands";

interface SettingsViewProps {
  config: AppConfigRedacted;
  preferences: WidgetPreferences;
}

export function SettingsView({ config, preferences }: SettingsViewProps) {
  const [polling, setPolling] = useState(config.polling_interval_seconds);
  const [staleAfter, setStaleAfter] = useState(config.stale_after_seconds);
  const [thresholds, setThresholds] = useState(config.alert_thresholds.join(", "));
  const [nativeNotifications, setNativeNotifications] = useState(
    config.native_notifications_enabled,
  );
  const [discordAlerts, setDiscordAlerts] = useState(config.discord_alerts_enabled);
  const [captureMode, setCaptureMode] = useState<CaptureMode>(config.capture_mode);
  const [webhook, setWebhook] = useState("");
  const [alwaysOnTop, setAlwaysOnTop] = useState(preferences.always_on_top);
  const [visibleOnLaunch, setVisibleOnLaunch] = useState(preferences.visible_on_launch);
  const [status, setStatus] = useState("");
  const [error, setError] = useState("");

  async function handleSave(event: FormEvent) {
    event.preventDefault();
    setError("");
    const parsedThresholds = thresholds
      .split(",")
      .map((value) => Number.parseInt(value.trim(), 10))
      .filter((value) => Number.isFinite(value));
    try {
      await updateConfig(
        {
          polling_interval_seconds: polling,
          stale_after_seconds: staleAfter,
          alert_thresholds: parsedThresholds,
          native_notifications_enabled: nativeNotifications,
          discord_alerts_enabled: discordAlerts,
          capture_mode: captureMode,
        },
        webhook ? { discord_webhook_url: webhook } : undefined,
      );
      await updateWidgetPreferences({
        always_on_top: alwaysOnTop,
        visible_on_launch: visibleOnLaunch,
      });
      setWebhook("");
      setStatus("Settings saved.");
    } catch (caught) {
      setError(errorMessage(caught));
    }
  }

  async function handleClearWebhook() {
    setError("");
    try {
      await updateConfig({}, { discord_webhook_url: "" });
      setWebhook("");
      setStatus("Discord webhook cleared.");
    } catch (caught) {
      setError(errorMessage(caught));
    }
  }

  async function handleTestWebhook() {
    setError("");
    try {
      const result = await testDiscordWebhook(webhook || undefined);
      setStatus(result.message);
    } catch (caught) {
      setError(errorMessage(caught));
    }
  }

  return (
    <main className="settings-shell">
      <header className="settings-header">
        <h1>Ida Settings</h1>
        <div className="settings-actions">
          <button
            className="secondary-button"
            type="button"
            onClick={openConfigDirectory}
          >
            <FolderOpen size={16} aria-hidden="true" />
            Open Folder
          </button>
          <button className="primary-button" form="settings-form" type="submit">
            <Save size={16} aria-hidden="true" />
            Save
          </button>
        </div>
      </header>

      <form id="settings-form" className="settings-content" onSubmit={handleSave}>
        <section className="settings-section">
          <h2>Refresh</h2>
          <div className="field-grid">
            <div className="field">
              <label htmlFor="polling">Polling interval</label>
              <input
                id="polling"
                min={60}
                max={86400}
                type="number"
                value={polling}
                onChange={(event) => setPolling(event.currentTarget.valueAsNumber)}
              />
            </div>
            <div className="field">
              <label htmlFor="stale">Stale after</label>
              <input
                id="stale"
                min={120}
                max={172800}
                type="number"
                value={staleAfter}
                onChange={(event) => setStaleAfter(event.currentTarget.valueAsNumber)}
              />
            </div>
          </div>
          <div className="field">
            <label htmlFor="capture-mode">Capture mode</label>
            <select
              id="capture-mode"
              value={captureMode}
              onChange={(event) =>
                setCaptureMode(event.currentTarget.value as CaptureMode)
              }
            >
              <option value="native_then_wsl">Native then WSL</option>
              <option value="native_only">Native only</option>
              <option value="wsl_only">WSL only</option>
            </select>
          </div>
        </section>

        <section className="settings-section">
          <h2>Alerts</h2>
          <div className="field">
            <label htmlFor="thresholds">Thresholds</label>
            <input
              id="thresholds"
              value={thresholds}
              onChange={(event) => setThresholds(event.currentTarget.value)}
            />
          </div>
          <label className="toggle-row">
            <span>Native notifications</span>
            <input
              checked={nativeNotifications}
              type="checkbox"
              onChange={(event) => setNativeNotifications(event.currentTarget.checked)}
            />
          </label>
          <label className="toggle-row">
            <span>Discord alerts</span>
            <input
              checked={discordAlerts}
              type="checkbox"
              onChange={(event) => setDiscordAlerts(event.currentTarget.checked)}
            />
          </label>
          <div className="field">
            <label htmlFor="webhook">Discord webhook</label>
            <input
              id="webhook"
              placeholder={
                config.discord_webhook_configured
                  ? "Saved webhook configured"
                  : "https://discord.com/api/webhooks/..."
              }
              type="password"
              value={webhook}
              onChange={(event) => setWebhook(event.currentTarget.value)}
            />
          </div>
          <div className="settings-actions">
            <button
              className="secondary-button"
              type="button"
              onClick={handleTestWebhook}
            >
              <Send size={16} aria-hidden="true" />
              Test
            </button>
            <button
              className="secondary-button"
              type="button"
              onClick={handleClearWebhook}
            >
              Clear
            </button>
          </div>
        </section>

        <section className="settings-section">
          <h2>Widget</h2>
          <label className="toggle-row">
            <span>Always on top</span>
            <input
              checked={alwaysOnTop}
              type="checkbox"
              onChange={(event) => setAlwaysOnTop(event.currentTarget.checked)}
            />
          </label>
          <label className="toggle-row">
            <span>Visible on launch</span>
            <input
              checked={visibleOnLaunch}
              type="checkbox"
              onChange={(event) => setVisibleOnLaunch(event.currentTarget.checked)}
            />
          </label>
        </section>

        <div className="status-line" role={error ? "alert" : "status"}>
          {error ? <span className="error-text">{error}</span> : status}
        </div>
      </form>
    </main>
  );
}

export function SettingsWindow() {
  const [config, setConfig] = useState<AppConfigRedacted | null>(null);
  const [preferences, setPreferences] = useState<WidgetPreferences | null>(null);

  useEffect(() => {
    void Promise.all([getConfig(), getWidgetPreferences()]).then(
      ([configResponse, preferencesResponse]) => {
        setConfig(configResponse.config);
        setPreferences(preferencesResponse.preferences);
      },
    );
  }, []);

  if (!config || !preferences) {
    return <main className="settings-shell" />;
  }

  return <SettingsView config={config} preferences={preferences} />;
}

function errorMessage(caught: unknown) {
  if (
    typeof caught === "object" &&
    caught !== null &&
    "message" in caught &&
    typeof caught.message === "string"
  ) {
    return caught.message;
  }
  return "Could not apply settings.";
}
