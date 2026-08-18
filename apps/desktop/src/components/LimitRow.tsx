import type { LimitStatus, LimitWindow } from "../bindings/ida";

const statusLabels: Record<LimitStatus, string> = {
  healthy: "Healthy",
  watch: "Watch",
  low: "Low",
  critical: "Critical",
  stale: "Stale",
  error: "Error",
};

interface LimitRowProps {
  limit: LimitWindow | null;
  label: string;
}

export function LimitRow({ limit, label }: LimitRowProps) {
  if (!limit) {
    return (
      <section className="limit-tile status-error" aria-label={`${label} missing`}>
        <div className="limit-row-top">
          <span className="limit-label">{label}</span>
          <span className="status-pill">Partial</span>
        </div>
        <div className="limit-percent">--</div>
        <div className="limit-reset">No current value</div>
      </section>
    );
  }

  const resetText =
    limit.raw_reset_text ||
    (limit.resets_at ? new Date(limit.resets_at).toLocaleString() : "Reset unknown");

  return (
    <section
      className={`limit-tile status-${limit.status}`}
      aria-label={`${limit.label} limit`}
    >
      <div className="limit-row-top">
        <span className="limit-label">{limit.label}</span>
        <span className="status-pill">{statusLabels[limit.status]}</span>
      </div>
      <div className="limit-percent">{limit.remaining_pct}%</div>
      <div className="limit-reset">{resetText}</div>
    </section>
  );
}
