# Ida Smoke Checklist

Run from the repository root on Windows PowerShell.

## Automated Checks

```powershell
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all --check
pnpm --dir apps/desktop typecheck
pnpm --dir apps/desktop test
pnpm --dir apps/desktop lint
pnpm --dir apps/desktop format:check
pnpm --dir apps/desktop tauri build
```

If `pnpm` is not on PATH, use `corepack pnpm` with the same arguments.

## Manual Runtime Smoke

- Launch with `pnpm --dir apps/desktop tauri dev`.
- Confirm the widget opens as a compact always-on-top window.
- Confirm the widget shows two primary rows, `5-hour` and `Weekly`, when fresh
  Codex fixture or live data is available.
- Trigger Refresh Now from the tray menu and confirm the widget updates or shows
  a short actionable error.
- Hide and show the widget from the tray; closing the widget should hide it
  while the tray app remains alive.
- Open Settings from the tray; save polling interval, stale threshold, capture
  mode, alert thresholds, notification toggles, Discord toggle, and widget
  visibility preferences.
- Save a Discord webhook only in local config, then confirm `get_config` and the
  settings UI show only secret presence or a masked placeholder.
- Use the Discord test command with a mock or disposable test endpoint. Never
  commit the URL.
- Force stale/error state with mocked provider failures and confirm the widget
  shows no fake percentages when no snapshot exists.
- Confirm native notification and Discord alert dedupe by crossing a threshold
  once, refreshing again in the same reset window, and observing no duplicate
  alert for the same channel/key.

## Prototype Boundary

The `local/` folder remains the Codex Limits prototype/reference
implementation. The protected files are `local/monitor.sh`,
`local/dashboard.html`, `local/data.json`, and `local/history.json`.
