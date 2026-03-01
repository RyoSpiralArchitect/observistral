param(
  [Parameter(ValueFromRemainingArguments = $true)]
  [string[]]$Args
)

$ErrorActionPreference = "Stop"

# On Windows, Cargo can't overwrite a running .exe. If `obstral.exe` is still
# running (e.g. from a previous TUI/serve session), `cargo run` fails with:
#   failed to remove file ...\\target\\debug\\obstral.exe (os error 5)
Get-Process obstral -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue

$repo = Resolve-Path (Join-Path $PSScriptRoot "..")
Push-Location $repo
try {
  cargo run -- tui @Args
} finally {
  Pop-Location
}

