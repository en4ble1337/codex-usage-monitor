# Ida Windows Usage Widget MVP

Ida is a Windows-first Tauri desktop widget for monitoring local Codex usage.
It keeps the existing `local/` Codex Limits prototype intact while new product
code lives in the Ida workspace:

```text
apps/desktop/            Tauri v2 + React widget/settings app
apps/desktop/src-tauri/  native Windows shell, commands, tray, notifications
core/ida-core/           provider-neutral models, storage, config, runtime, alerts
providers/codex/         Codex capture and parser implementation
providers/claude/        placeholder only; no MVP implementation
docs/                    PRD, architecture, research, smoke notes
directives/              implementation work orders and status notes
execution/               repeatable local smoke scripts
tests/                   shared sanitized fixtures and future integration tests
local/                   existing Codex Limits prototype/reference implementation
.tmp/                    gitignored scratch space
```

`local/monitor.sh`, `local/dashboard.html`, `local/data.json`, and
`local/history.json` remain prototype/reference files. Do not add new Ida
product code to `local/`.

## Ida Prerequisites

- Windows 10/11 for the MVP desktop target.
- Rust stable 1.82 or newer.
- Node 24 LTS preferred, Node 22.12 or newer minimum.
- pnpm 10.x. If `pnpm` is not on PATH, use Corepack:
  `corepack prepare pnpm@10.20.0 --activate`, or prefix commands with
  `corepack pnpm`.
- Tauri v2 Windows prerequisites, including Microsoft Visual Studio Build Tools
  with the MSVC C++ workload and WebView2.
- Codex CLI installed and authenticated. Ida invokes local Codex status capture
  and never handles OpenAI credentials.
- WSL is optional, but required when using `native_then_wsl` fallback or
  `wsl_only` capture mode on a setup where native Codex is unavailable.

## Run Ida From Source

Install dependencies:

```powershell
corepack pnpm install
```

Run the desktop app in development:

```powershell
pnpm --dir apps/desktop tauri dev
```

If `pnpm` is unavailable in this shell, use:

```powershell
corepack pnpm --dir apps/desktop tauri dev
```

Build a local Windows package:

```powershell
pnpm --dir apps/desktop tauri build
```

The MVP has no app account, cloud backend, license enforcement, payment
processing, or auto-update flow. Future paid builds, if offered, are convenience
packages for users who do not want to compile from source.

## Ida Verification

Run the full local smoke script:

```powershell
.\execution\smoke.ps1
```

Or run the checks directly:

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

The commands above should not require secrets. Do not commit config files,
Discord webhook URLs, API keys, raw terminal transcripts, snapshots, logs, build
outputs, `target/`, `node_modules/`, or generated installers.

# Codex Usage Monitor Prototype

Track your OpenAI Codex CLI usage limits in real time locally, with optional external access. No cloud accounts required to get started.

**Problem:** OpenAI already lets you check Codex usage in `codex /status` and in the ChatGPT Codex usage page, but those views do not give you rolling history or proactive notifications when you're running low.

**Solution:** A local bash script that scrapes `codex /status` every 15 minutes, writes a `data.json` snapshot, serves a dashboard in your browser, and fires Discord or Telegram alerts directly — all from your own machine, with zero cloud dependency.

![Codex Limits dashboard hero](local/images/hero.png)

---

## Table of Contents

1. [Architecture Overview](#architecture-overview)
2. [Project Structure](#project-structure)
3. [Prerequisites](#prerequisites)
4. [Quick Start (Local)](#quick-start-local)
5. [Ways To Run It](#ways-to-run-it)
   - [Option A: Two Local Terminals](#option-a-two-local-terminals-easiest)
   - [Option B: Bash Background Process](#option-b-bash-background-process)
   - [Option C: tmux](#option-c-tmux-recommended)
   - [Option D: WSL (Windows)](#option-d-wsl-windows)
   - [Option E: LXC Container](#option-e-lxc-container)
   - [Option F: systemd service](#option-f-systemd-service-linux)
   - [Option G: cron](#option-g-cron)
6. [Notifications (Discord & Telegram)](#notifications-discord--telegram)
7. [External Dashboard (Optional)](#external-dashboard-optional-github-gist--pages)
8. [Configuration Reference](#configuration-reference)
9. [Troubleshooting](#troubleshooting)
10. [Contributing](#contributing)
11. [License](#license)

---

## Architecture Overview

![Codex Limits framework](local/images/framework.png)

```
┌──────────────────────────────────────┐
│  Your machine (any OS with bash)      │
│                                        │
│  codex /status                         │
│       ↓                                │
│  local/monitor.sh                      │
│       ↓              ↓                 │
│  local/data.json   Discord/Telegram    │
│  local/history.json  (direct curl)     │
│       ↓                                │
│  local/dashboard.html                  │
│  (browser via serve.sh)                │
└──────────────────────────────────────┘
          │ optional (Tier 2)
          ▼
   GitHub Gist ──→ yourname.github.io/codex-monitor
   (JSON blob)       (static dashboard)
```

**Tier 1 — Local only:** Everything runs on your machine. No external accounts beyond your existing OpenAI / ChatGPT subscription.  
**Tier 2 — External dashboard (optional):** `monitor.sh` also PATCHes a GitHub Gist. A static GitHub Pages site reads from it. Requires a GitHub account and a Personal Access Token.

---

## Project Structure

```text
codex-usage-monitor/
|-- local/
|   |-- monitor.sh        # Scraper, parser, alerts, optional Gist sync
|   |-- dashboard.html    # Local dashboard UI
|   |-- serve.sh          # Simple HTTP server for the dashboard
|   |-- .env.example      # Config template; copy to .env
|   |-- .env              # Your local config (git-ignored, created by you)
|   |-- data.json         # Latest usage snapshot written by monitor.sh
|   |-- history.json      # Rolling history used by the chart
|   `-- images/
|       |-- hero.png
|       |-- framework.png
|       |-- discord.png
|       |-- logo.png
|-- .gitignore
|-- LICENSE
`-- README.md
```

---

## Prerequisites

### Required (Tier 1)
| Requirement | Check | Install / Setup |
|---|---|---|
| OpenAI Codex CLI | `codex --version` | `npm i -g @openai/codex` — [CLI docs](https://developers.openai.com/codex/cli/) |
| Codex CLI authenticated | `codex /status` shows usage data | Run `codex` once and sign in (see note below) |
| bash | `bash --version` | Pre-installed on Linux/WSL |
| curl | `curl --version` | Pre-installed on most systems |
| python3 | `python3 --version` | [python.org](https://python.org) — needed for dashboard server & JSON handling |
| GNU grep | `grep -P '' /dev/null` | Built-in on most Linux distributions |

> [!NOTE]
> **First-time Codex users — you must authenticate before using this tool.**
> 
> The monitor scrapes `codex /status`, which only works after you have signed in. If you have never run Codex before:
> 
> ```bash
> # 1. Install the CLI
> npm i -g @openai/codex
> 
> # 2. Launch Codex — it will prompt you to sign in on first run
> codex
> ```
> 
> Authenticate with your **ChatGPT account** (Plus, Pro, Business, Edu, or Enterprise) or an **API key**.
> Once signed in, verify with `codex /status` — you should see your usage percentages.
> 
> Full setup details: [developers.openai.com/codex/cli](https://developers.openai.com/codex/cli/)

> **Windows users:** Run everything inside WSL. Native Windows bash is not supported. See the [Windows setup guide](https://developers.openai.com/codex/windows).
>
> **Supported / tested:** Linux-based shells only. We have tested this on WSL and on a Proxmox LXC container.

### Optional (Tier 2 — External Dashboard)
| Requirement | Notes |
|---|---|
| GitHub account | Free — for Gist storage and Pages hosting |
| GitHub Personal Access Token | `gist` scope only. Create at [github.com/settings/tokens](https://github.com/settings/tokens) |

---

## Quick Start (Local)

```bash
# 0. Make sure Codex CLI is installed and authenticated
#    (skip if you already use Codex daily)
npm i -g @openai/codex   # install
codex                     # first run — sign in when prompted
codex /status             # verify — should show usage percentages

# 1. Clone
git clone https://github.com/YOUR_USERNAME/codex-usage-monitor.git
cd codex-usage-monitor/local

# 2. Configure
cp .env.example .env
# Edit .env — Discord/Telegram are optional, leave blank to skip

# 3. Make executable
chmod +x monitor.sh serve.sh

# 4. Run once to verify
./monitor.sh
# Output: parsed JSON printed to terminal, data.json written

# 5. Open the dashboard
./serve.sh
# Then open: http://localhost:8080/dashboard.html
```

Important: `serve.sh` only hosts the dashboard files. It does not refresh usage by itself. For live updates, `monitor.sh` must also be running on a loop or schedule.

---

## Ways To Run It

Pick the simplest option that fits your environment. In every setup:

- `monitor.sh` scrapes Codex and writes `data.json` / `history.json`
- `serve.sh` serves `dashboard.html`

This project is designed for local Linux-style execution. Docker is intentionally not documented as a supported runtime because the current status capture depends on an authenticated local Codex CLI environment.

### Option A: Two Local Terminals (easiest)

Best for: first-time setup, testing, and most local users on Linux/WSL.

Terminal 1:
```bash
cd /path/to/codex-usage-monitor/local
./monitor.sh --loop 900
```

Terminal 2:
```bash
cd /path/to/codex-usage-monitor/local
./serve.sh
```

Then open:
```text
http://localhost:8080/dashboard.html
```

For faster testing:
```bash
./monitor.sh --loop 60
```

---

### Option B: Bash Background Process

Best for: one shell, lightweight local use, quick demos.

```bash
cd /path/to/codex-usage-monitor/local
./monitor.sh --loop 900 > /tmp/codex-monitor.log 2>&1 &
MONITOR_PID=$!
./serve.sh
```

Stop the background monitor later with:
```bash
kill "$MONITOR_PID"
```

---

### Option C: tmux (recommended)

Best for: any Linux/WSL machine where you want a simple persistent session.

```bash
# Install if needed
sudo apt install tmux        # Debian/Ubuntu

# Start monitor in a named session
tmux new -s codex-monitor 'cd /path/to/codex-usage-monitor/local && ./monitor.sh --loop 900'

# Detach (leave running): Ctrl+B, then D
# Reattach later:
tmux attach -t codex-monitor

# Start dashboard in a second session (optional)
tmux new -s codex-dash 'cd /path/to/codex-usage-monitor/local && ./serve.sh'
```

---

### Option D: WSL (Windows)

Best for: Windows users who want native Linux tooling without a full VM.

```powershell
# Install WSL2 (run in PowerShell as Administrator)
wsl --install

# Then in the WSL terminal:
cd /mnt/c/path/to/codex-usage-monitor/local
chmod +x monitor.sh serve.sh
./monitor.sh            # test once

# Run continuously with tmux inside WSL
tmux new -s codex-monitor './monitor.sh --loop 900'
```

> The dashboard served from WSL is accessible on Windows via `http://localhost:8080/dashboard.html` — WSL2 bridges the network automatically.

---

### Option E: LXC Container

Best for: Proxmox homelab users or anyone running LXC on Linux.

```bash
# Create a lightweight Alpine or Ubuntu container
# (example using Proxmox pct or plain lxc-create)

# Proxmox:
pct create 200 local:vztmpl/ubuntu-22.04-standard_22.04-1_amd64.tar.zst \
  --hostname codex-monitor --memory 256 --rootfs local-lvm:4

pct start 200
pct exec 200 -- bash

# Inside the container:
apt update && apt install -y bash curl python3 git grep
git clone https://github.com/YOUR_USERNAME/codex-usage-monitor.git
cd codex-usage-monitor/local
cp .env.example .env && nano .env
chmod +x monitor.sh serve.sh

# Run monitor as systemd service (see Option F below)
# Bind port 8080 in your LXC config if you want external access:
# pct set 200 -net0 name=eth0,bridge=vmbr0,ip=dhcp
```

---

### Option F: systemd service (Linux)

Best for: servers, headless Linux boxes, Raspberry Pi — auto-starts on boot and restarts on failure.

```bash
# Create service file (adjust paths)
sudo tee /etc/systemd/system/codex-monitor.service << 'EOF'
[Unit]
Description=Codex Usage Monitor
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
User=youruser
WorkingDirectory=/home/youruser/codex-usage-monitor/local
EnvironmentFile=/home/youruser/codex-usage-monitor/local/.env
ExecStart=/bin/bash /home/youruser/codex-usage-monitor/local/monitor.sh --loop 900
Restart=on-failure
RestartSec=30

[Install]
WantedBy=multi-user.target
EOF

# Enable and start
sudo systemctl daemon-reload
sudo systemctl enable --now codex-monitor

# Check status and logs
sudo systemctl status codex-monitor
journalctl -u codex-monitor -f
```

**Add a second service for the dashboard:**
```bash
sudo tee /etc/systemd/system/codex-dashboard.service << 'EOF'
[Unit]
Description=Codex Usage Dashboard
After=network.target

[Service]
Type=simple
User=youruser
ExecStart=/bin/bash /home/youruser/codex-usage-monitor/local/serve.sh
Restart=on-failure

[Install]
WantedBy=multi-user.target
EOF

sudo systemctl enable --now codex-dashboard
```

---

### Option G: cron

Best for: lightweight — no persistent process, just runs on schedule.

```bash
crontab -e

# Add this line (adjust path):
*/15 * * * * /bin/bash /home/youruser/codex-usage-monitor/local/monitor.sh >> /tmp/codex-monitor.log 2>&1
```

---

## Notifications (Discord & Telegram)

Both work via a **direct `curl` call from `monitor.sh`** — no server, no middleman, no third-party backend. As long as your machine has internet access, alerts fire.

### Discord

1. Open your Discord server → **Server Settings** → **Integrations** → **Webhooks**
2. Click **New Webhook**, pick a channel, click **Copy Webhook URL**
3. Add to `local/.env`:
   ```bash
   DISCORD_WEBHOOK=https://discord.com/api/webhooks/123456789/abcdefgh...
   ```

**Test it directly (no monitor needed):**
```bash
source local/.env
curl -X POST "$DISCORD_WEBHOOK" \
  -H "Content-Type: application/json" \
  -d '{"content": "✅ Codex monitor test alert"}'
```

**Example Discord alert:**

![Discord alert proof](local/images/discord.png)

---

### Telegram

1. Open Telegram → message **@BotFather** → send `/newbot` → follow prompts → copy the **bot token**
2. Start a chat with your new bot (send it any message)
3. Find your chat ID:
   ```bash
   curl "https://api.telegram.org/bot<YOUR_TOKEN>/getUpdates"
   # Look for "chat":{"id": <number>} in the response
   ```
4. Add to `local/.env`:
   ```bash
   TELEGRAM_BOT_TOKEN=123456789:ABCdefGHI-jklMNO
   TELEGRAM_CHAT_ID=987654321
   ```

**Test it directly:**
```bash
source local/.env
curl -X POST "https://api.telegram.org/bot${TELEGRAM_BOT_TOKEN}/sendMessage" \
  -d "chat_id=${TELEGRAM_CHAT_ID}" \
  --data-urlencode "text=✅ Codex monitor test alert"
```

---

### Alert Thresholds

Alerts fire **once** as usage drops below each threshold — no spam.

```bash
# Default: alert at 75%, 50%, 25%, 10%, 5% remaining
ALERT_THRESHOLDS=75,50,25,10,5

# Minimal alerting
ALERT_THRESHOLDS=25,5

# Verbose
ALERT_THRESHOLDS=90,75,50,25,10,5,1
```

---

## External Dashboard (Optional — GitHub Gist + Pages)

Skip this section entirely if you only need the local dashboard.

This tier lets you view the dashboard from any browser anywhere, using:
- **GitHub Gist** as a free JSON data store (updated by `monitor.sh` via `curl`)
- **GitHub Pages** to host `local/dashboard.html` — the same file used locally, just deployed statically

### Setup (one-time, ~10 minutes)

**Step 1 — Create a Gist**

1. Go to [gist.github.com](https://gist.github.com)
2. Create a **secret** Gist with a file named `data.json` (contents can be `{}` for now)
3. Copy the Gist ID from the URL: `gist.github.com/<username>/<GIST_ID>`

**Step 2 — Create a Personal Access Token**

1. Go to [github.com/settings/tokens](https://github.com/settings/tokens) → **Generate new token (classic)**
2. Check only the **`gist`** scope
3. Copy the token

**Step 3 — Configure `local/.env`**

```bash
GITHUB_PAT=ghp_yourTokenHere
GITHUB_GIST_ID=abc123def456...
```

Run `./monitor.sh` once — you should see `[OK] Gist updated` in the output.

**Step 4 — Set your Gist ID in `local/dashboard.html`**

Open `local/dashboard.html` and set `GIST_ID` near the top of the `<script>` block:

```javascript
const GIST_ID = 'abc123def456...';  // your actual Gist ID
```

With this set, the same `dashboard.html` file switches into **external mode** and fetches data from your Gist instead of local files.

**Step 5 — Deploy `dashboard.html` to GitHub Pages**

Option A — Add to an existing GitHub Pages repo:
```bash
cp local/dashboard.html ~/my-pages-repo/codex/index.html
cd ~/my-pages-repo && git add . && git commit -m "Add Codex monitor" && git push
# Access at: https://yourname.github.io/codex/
```

Option B — Enable Pages on this repo:
1. Push this repo to GitHub
2. Go to **Settings → Pages → Source → Deploy from branch**
3. Select `main` branch, `/local` folder
4. Rename `dashboard.html` → `index.html` for a cleaner URL
5. Access at: `https://yourname.github.io/codex-usage-monitor/`

---

## Configuration Reference

All variables go in `local/.env` (copy from `local/.env.example`).

| Variable | Required | Default | Description |
|---|---|---|---|
| `DISCORD_WEBHOOK` | No | — | Discord webhook URL |
| `TELEGRAM_BOT_TOKEN` | No | — | Telegram bot token from BotFather |
| `TELEGRAM_CHAT_ID` | No | — | Numeric Telegram chat ID |
| `ALERT_THRESHOLDS` | No | `75,50,25,10,5` | Comma-separated % thresholds for alerts |
| `GITHUB_PAT` | No (Tier 2 only) | — | GitHub Personal Access Token (`gist` scope) |
| `GITHUB_GIST_ID` | No (Tier 2 only) | — | ID of the Gist to update |

---

## Troubleshooting

### `codex: command not found`

The Codex CLI is not in PATH. Run `which codex` or check your shell profile. Try running `codex /status` manually first.

### `Could not parse usage percentages from codex output`

OpenAI may have changed the `codex /status` output format. Run `codex /status 2>&1 | cat` to see raw output and check whether `5h limit:` / `Weekly limit:` lines still appear with `% left`.

If you are running inside WSL and `codex /status` gets treated like a normal prompt, update to the current `local/monitor.sh`. It now opens Codex in a PTY and sends `/status` as terminal input instead of passing it as a CLI argument. If Codex starts slowly in your environment, raise `CODEX_STATUS_TIMEOUT_SECONDS` in `local/.env`.

### `grep -P` errors on macOS

Install GNU grep:
```bash
brew install grep
export PATH="/opt/homebrew/bin:$PATH"  # add to ~/.zshrc
```

### Dashboard shows blank / "Could not load data.json"

`monitor.sh` hasn't run yet, or you opened `dashboard.html` directly from the filesystem (not via `serve.sh`). Run:
```bash
./monitor.sh          # creates data.json
./serve.sh            # starts http server
# then open http://localhost:8080/dashboard.html
```

### Gist sync returns HTTP 401

Your `GITHUB_PAT` is expired or missing `gist` scope. Generate a new token at [github.com/settings/tokens](https://github.com/settings/tokens).

### Gist sync returns HTTP 404

`GITHUB_GIST_ID` is wrong. Double-check the ID from the Gist URL.

---


## Contributing

PRs welcome. Some ideas:

- [ ] Detect `codex /status --json` if/when OpenAI adds it
- [ ] Slack webhook support
- [ ] ntfy.sh support (self-hosted push notifications)
- [ ] Multi-account support
- [ ] Longer history retention (configurable window)
- [ ] Email alerts via a simple SMTP relay
- [ ] macOS-compatible parsing (pure sed, no GNU grep)
- [ ] Auto-open browser on `serve.sh` start
- [ ] GitHub Actions workflow for automated Gist update (no local machine needed)

---

## Local Testing Notes

- History retention is time-based now. `HISTORY_RETENTION_HOURS=24` keeps a 24-hour window even if you change the scrape interval.
- Docker should only be used when the container can access an authenticated Codex CLI config. A common setup is mounting `~/.codex` into `/root/.codex`. If `codex /status` does not work on the host, it will not work inside Docker either — authenticate first.
- The container now performs a startup scrape and exits if monitoring cannot start, so it will not keep serving stale data after the scraper dies.
- The dashboard clears metrics and marks itself stale on refresh failure instead of leaving old percentages visible.


## License

MIT
