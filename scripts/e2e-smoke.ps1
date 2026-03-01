Param(
  [Alias("Host")]
  [string]$ListenHost = "127.0.0.1",
  [int]$Port = 18090,
  [string]$WorkspaceRoot = ""
)

$ErrorActionPreference = "Stop"

$repoRoot = Resolve-Path (Join-Path $PSScriptRoot "..")
Set-Location $repoRoot

$ws = if ($WorkspaceRoot -and $WorkspaceRoot.Trim()) {
  $WorkspaceRoot.Trim()
} else {
  Join-Path (Resolve-Path ".tmp") "e2e-smoke-work"
}

New-Item -ItemType Directory -Force -Path $ws | Out-Null

Write-Host "Starting Lite server: http://$ListenHost`:$Port/  workspace=$ws"
$p = Start-Process -FilePath python -ArgumentList @(
  "scripts/serve_lite.py",
  "--host", $ListenHost,
  "--port", $Port,
  "--workspace", $ws
) -PassThru -WindowStyle Hidden

try {
  Start-Sleep -Milliseconds 800

  $st = Invoke-RestMethod -Method Get -Uri "http://$ListenHost`:$Port/api/status" -TimeoutSec 5
  if (-not $st.ok) { throw "status.ok is false" }

  $tid = "thread_e2e_smoke"
  $cwd = ".tmp/$tid"
  $body = @{
    command = "New-Item -ItemType Directory -Force -Path 'demo-repo' | Out-Null; Set-Location 'demo-repo'; git init"
    cwd = $cwd
    timeout_seconds = 60
  } | ConvertTo-Json

  $ex = Invoke-RestMethod -Method Post -Uri "http://$ListenHost`:$Port/api/exec" -ContentType "application/json" -Body $body -TimeoutSec 15
  if ($ex.exit_code -ne 0) {
    throw "exec failed: exit_code=$($ex.exit_code) stderr=$($ex.stderr)"
  }

  $gitDir = Join-Path (Join-Path (Join-Path $ws $cwd) "demo-repo") ".git"
  if (-not (Test-Path $gitDir)) {
    throw "expected git dir missing: $gitDir"
  }

  Write-Host "E2E smoke OK: $gitDir"
  exit 0
} finally {
  try { Stop-Process -Id $p.Id -Force } catch {}
}

