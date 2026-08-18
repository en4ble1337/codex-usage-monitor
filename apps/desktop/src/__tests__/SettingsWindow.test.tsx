import { fireEvent, render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { AppConfigRedacted, WidgetPreferences } from "../bindings/ida";
import { SettingsView } from "../windows/SettingsWindow";

vi.mock("../commands", () => ({
  updateConfig: vi.fn().mockResolvedValue({
    config: {},
    restart_required: false,
  }),
  updateWidgetPreferences: vi.fn().mockResolvedValue({
    preferences: {},
  }),
  testDiscordWebhook: vi.fn().mockResolvedValue({
    delivery_status: "sent",
    status_code: 204,
    message: "Discord test alert sent.",
  }),
  openConfigDirectory: vi.fn(),
}));

const config: AppConfigRedacted = {
  schema_version: 1,
  active_provider_id: "codex",
  polling_interval_seconds: 900,
  stale_after_seconds: 1800,
  alert_thresholds: [75, 50, 25, 10, 5],
  native_notifications_enabled: true,
  discord_alerts_enabled: false,
  discord_webhook_configured: true,
  discord_webhook_masked: "https://discord.com/api/webhooks/...",
  capture_mode: "native_then_wsl",
  history_retention_hours: 24,
  log_level: "info",
  config_path: "C:/Users/Example/AppData/Roaming/Ida/config.json",
};

const preferences: WidgetPreferences = {
  schema_version: 1,
  visible_on_launch: true,
  always_on_top: true,
  position_x: null,
  position_y: null,
  width: 280,
  height: 160,
  display_id: null,
};

describe("SettingsView", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("masks saved Discord webhook values", () => {
    render(<SettingsView config={config} preferences={preferences} />);

    const webhook = screen.getByLabelText("Discord webhook");
    expect(webhook).toHaveAttribute("type", "password");
    expect(webhook).toHaveAttribute("placeholder", "Saved webhook configured");
    expect(webhook).toHaveValue("");
  });

  it("saves polling and widget preferences", async () => {
    const user = userEvent.setup();
    render(<SettingsView config={config} preferences={preferences} />);

    fireEvent.change(screen.getByLabelText("Polling interval"), {
      target: { value: "120" },
    });
    await user.click(screen.getByRole("button", { name: /^save$/i }));

    expect(await screen.findByText("Settings saved.")).toBeInTheDocument();
  });

  it("can test a transient webhook without displaying it after save", async () => {
    const user = userEvent.setup();
    render(<SettingsView config={config} preferences={preferences} />);

    await user.type(
      screen.getByLabelText("Discord webhook"),
      "https://discord.com/api/webhooks/12345/transient",
    );
    await user.click(screen.getByRole("button", { name: /test/i }));

    expect(await screen.findByText("Discord test alert sent.")).toBeInTheDocument();
  });
});
