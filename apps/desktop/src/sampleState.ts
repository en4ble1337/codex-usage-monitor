import type { AppState, LimitWindow } from "./bindings/ida";

const now = new Date().toISOString();

function limit(
  id: "5h" | "weekly",
  label: string,
  remaining_pct: number,
  raw_reset_text: string,
): LimitWindow {
  const status =
    remaining_pct >= 50
      ? "healthy"
      : remaining_pct >= 25
        ? "watch"
        : remaining_pct >= 10
          ? "low"
          : "critical";
  return {
    id,
    label,
    window: id,
    remaining_pct,
    used_pct: 100 - remaining_pct,
    resets_at: null,
    raw_reset_text,
    status,
    status_reason: null,
    metadata: {},
  };
}

export const sampleAppState: AppState = {
  schema_version: 1,
  provider_id: "codex",
  latest_snapshot: {
    schema_version: 1,
    provider_id: "codex",
    provider_name: "Codex",
    provider_status: "ok",
    scraped_at: now,
    capture_method: "fixture",
    source_platform: "windows",
    limits: [
      limit("5h", "5-hour", 89, "in 2h 14m"),
      limit("weekly", "Weekly", 64, "Monday 09:00"),
    ],
    metadata: {
      account_label: null,
      raw_model_label: "GPT-5 Codex",
      parser_version: "codex-status-v1",
      raw_fields: { source: "fixture" },
    },
  },
  current_error: null,
  freshness_status: "fresh",
  last_attempted_at: now,
  last_success_at: now,
  next_poll_at: null,
  effective_limits: [
    limit("5h", "5-hour", 89, "in 2h 14m"),
    limit("weekly", "Weekly", 64, "Monday 09:00"),
  ],
};
