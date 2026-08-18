import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { RefreshCw } from "lucide-react";
import { useCallback, useEffect, useMemo, useState } from "react";
import type { AppError, AppState, LimitWindow } from "../bindings/ida";
import { LimitRow } from "../components/LimitRow";
import { getAppState, refreshUsage, updateWidgetPreferences } from "../commands";

const errorHints: Partial<Record<AppError["code"], string>> = {
  CODEX_NOT_FOUND: "Install Codex or update PATH, then refresh.",
  CODEX_UNAUTHENTICATED: "Open Codex once and sign in.",
  WSL_NOT_FOUND: "Install WSL or switch capture mode.",
  WSL_UNAVAILABLE: "Check the selected WSL distro.",
  PARSER_FAILED: "Codex output changed. Refresh after updating Ida.",
  SNAPSHOT_CORRUPT: "Open config and remove the corrupt snapshot.",
};

interface WidgetViewProps {
  state: AppState;
  loading?: boolean;
  onRefresh?: () => void;
  onDragStart?: () => void;
}

export function WidgetView({
  state,
  loading = false,
  onRefresh,
  onDragStart,
}: WidgetViewProps) {
  const limits = useMemo(() => orderLimits(state.effective_limits), [state]);
  const hasLimits = limits.some(Boolean);
  const lastSuccess = state.last_success_at
    ? new Date(state.last_success_at).toLocaleTimeString([], {
        hour: "numeric",
        minute: "2-digit",
      })
    : null;
  const stateLabel =
    state.freshness_status === "fresh"
      ? "Fresh"
      : state.freshness_status === "stale"
        ? "Stale"
        : "Needs setup";

  return (
    <main className="widget-shell">
      <header className="widget-titlebar" onPointerDown={onDragStart}>
        <div className="widget-title">Ida</div>
        <div className="widget-actions">
          <button
            className="icon-button"
            aria-label="Refresh usage"
            onPointerDown={(event) => event.stopPropagation()}
            onClick={onRefresh}
            type="button"
          >
            <RefreshCw size={16} aria-hidden="true" />
          </button>
        </div>
      </header>

      {hasLimits ? (
        <div className="limits-grid">
          <LimitRow limit={limits[0]} label="5-hour" />
          <LimitRow limit={limits[1]} label="Weekly" />
        </div>
      ) : (
        <EmptyState error={state.current_error} />
      )}

      <footer className="widget-footer">
        <span>{loading ? "Refreshing" : stateLabel}</span>
        <span>{lastSuccess ? `Last ${lastSuccess}` : "No snapshot"}</span>
      </footer>
    </main>
  );
}

export function WidgetWindow() {
  const [state, setState] = useState<AppState | null>(null);
  const [loading, setLoading] = useState(true);

  const load = useCallback(async () => {
    const response = await getAppState(false);
    setState(response.state);
  }, []);

  useEffect(() => {
    void load().finally(() => setLoading(false));
    const unlisten = listen<AppState>("ida:state-changed", (event) => {
      setState(event.payload);
    }).catch(() => undefined);
    return () => {
      void unlisten.then((cleanup) => cleanup?.());
    };
  }, [load]);

  const handleRefresh = async () => {
    setLoading(true);
    try {
      const outcome = await refreshUsage("manual");
      setState(outcome.state);
    } finally {
      setLoading(false);
    }
  };

  const handleDragStart = async () => {
    try {
      await getCurrentWindow().startDragging();
      await persistBounds();
    } catch {
      // Browser preview does not expose Tauri window APIs.
    }
  };

  if (!state) {
    return <WidgetView state={emptyState()} loading={loading} />;
  }

  return (
    <WidgetView
      state={state}
      loading={loading}
      onRefresh={handleRefresh}
      onDragStart={handleDragStart}
    />
  );
}

function orderLimits(limits: LimitWindow[]): [LimitWindow | null, LimitWindow | null] {
  return [
    limits.find((limit) => limit.id === "5h") ?? null,
    limits.find((limit) => limit.id === "weekly") ?? null,
  ];
}

function EmptyState({ error }: { error: AppError | null }) {
  const hint = error ? (errorHints[error.code] ?? "Refresh again in a moment.") : null;
  return (
    <section className="empty-state" aria-label="No usage data">
      <strong>{error ? error.message : "No Codex usage yet"}</strong>
      <span>{hint ?? "Refresh after Codex is installed and signed in."}</span>
    </section>
  );
}

async function persistBounds() {
  const window = getCurrentWindow();
  const [position, size] = await Promise.all([
    window.outerPosition(),
    window.outerSize(),
  ]);
  await updateWidgetPreferences({
    position_x: position.x,
    position_y: position.y,
    width: size.width,
    height: size.height,
  });
}

function emptyState(): AppState {
  return {
    schema_version: 1,
    provider_id: "codex",
    latest_snapshot: null,
    current_error: null,
    freshness_status: "unavailable",
    last_attempted_at: null,
    last_success_at: null,
    next_poll_at: null,
    effective_limits: [],
  };
}
