import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import type { AppState, LimitWindow } from "../bindings/ida";
import { sampleAppState } from "../sampleState";
import { WidgetView } from "../windows/WidgetWindow";

function withLimits(limits: LimitWindow[], freshness = "fresh"): AppState {
  return {
    ...sampleAppState,
    freshness_status: freshness as AppState["freshness_status"],
    effective_limits: limits,
  };
}

function makeLimit(
  id: "5h" | "weekly",
  remaining_pct: number,
  status: LimitWindow["status"],
): LimitWindow {
  return {
    id,
    label: id === "5h" ? "5-hour" : "Weekly",
    window: id,
    remaining_pct,
    used_pct: 100 - remaining_pct,
    resets_at: null,
    raw_reset_text: id === "5h" ? "in 2h" : "Friday",
    status,
    status_reason: null,
    metadata: {},
  };
}

describe("WidgetView", () => {
  it("renders exactly two primary fresh limit rows", () => {
    render(<WidgetView state={sampleAppState} />);

    expect(screen.getByLabelText("5-hour limit")).toHaveTextContent("89%");
    expect(screen.getByLabelText("Weekly limit")).toHaveTextContent("64%");
    expect(screen.getAllByText(/Healthy/i)).toHaveLength(2);
  });

  it("renders watch, low, and critical status labels", () => {
    render(
      <WidgetView
        state={withLimits([
          makeLimit("5h", 25, "watch"),
          makeLimit("weekly", 9, "critical"),
        ])}
      />,
    );

    expect(screen.getByText("Watch")).toBeInTheDocument();
    expect(screen.getByText("Critical")).toBeInTheDocument();
  });

  it("shows stale last-known values with last successful time", () => {
    render(
      <WidgetView
        state={{
          ...withLimits(
            [makeLimit("5h", 42, "stale"), makeLimit("weekly", 12, "stale")],
            "stale",
          ),
          last_success_at: "2026-05-03T18:15:00Z",
        }}
      />,
    );

    expect(screen.getAllByText("Stale")).toHaveLength(3);
    expect(screen.getByText(/Last/)).toBeInTheDocument();
  });

  it("does not render fake percentages when no snapshot exists", () => {
    render(
      <WidgetView
        state={{
          ...sampleAppState,
          latest_snapshot: null,
          effective_limits: [],
          current_error: {
            code: "CODEX_UNAUTHENTICATED",
            message: "Codex is installed but not signed in.",
            details: {},
            operation_id: "00000000-0000-4000-8000-000000000000",
            occurred_at: "2026-05-03T18:15:00Z",
            retryable: true,
          },
          freshness_status: "error",
        }}
      />,
    );

    expect(screen.getByLabelText("No usage data")).toHaveTextContent(
      "Open Codex once and sign in.",
    );
    expect(screen.queryByText("--")).not.toBeInTheDocument();
  });

  it("marks a missing partial limit without hiding the valid one", () => {
    render(<WidgetView state={withLimits([makeLimit("5h", 42, "watch")])} />);

    expect(screen.getByLabelText("5-hour limit")).toHaveTextContent("42%");
    expect(screen.getByLabelText("Weekly missing")).toHaveTextContent("Partial");
  });
});
