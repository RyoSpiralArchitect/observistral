param(
  [string[]]$Args
)

$ErrorActionPreference = "Stop"

# Prevent the common Windows dev failure:
# `cargo run` cannot overwrite `target\\debug\\obstral.exe` if it's still running.
& (Join-Path $PSScriptRoot "kill-obstral.ps1") | Out-Null

Write-Host "[run-tui] cargo run -- tui" -ForegroundColor Cyan
if ($Args -and $Args.Count -gt 0) {
  cargo run -- tui @Args
} else {
  cargo run -- tui
}

