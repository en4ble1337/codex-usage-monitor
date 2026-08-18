param(
    [switch]$SkipTauriBuild
)

$ErrorActionPreference = "Stop"

function Invoke-Step {
    param(
        [string]$Name,
        [scriptblock]$Command
    )

    Write-Host "==> $Name"
    & $Command
}

$pnpm = "pnpm"
if (-not (Get-Command pnpm -ErrorAction SilentlyContinue)) {
    $pnpm = "corepack pnpm"
}

Invoke-Step "cargo check" { cargo check --workspace }
Invoke-Step "cargo test" { cargo test --workspace }
Invoke-Step "cargo clippy" { cargo clippy --workspace --all-targets -- -D warnings }
Invoke-Step "cargo fmt check" { cargo fmt --all --check }
Invoke-Step "frontend typecheck" { Invoke-Expression "$pnpm --dir apps/desktop typecheck" }
Invoke-Step "frontend test" { Invoke-Expression "$pnpm --dir apps/desktop test" }
Invoke-Step "frontend lint" { Invoke-Expression "$pnpm --dir apps/desktop lint" }
Invoke-Step "frontend format check" { Invoke-Expression "$pnpm --dir apps/desktop format:check" }

if (-not $SkipTauriBuild) {
    Invoke-Step "tauri build" { Invoke-Expression "$pnpm --dir apps/desktop tauri build" }
}
