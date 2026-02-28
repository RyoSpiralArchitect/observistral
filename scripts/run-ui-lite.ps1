Param(
  [Alias("Host")]
  [string]$ListenHost = "127.0.0.1",
  [int]$Port = 8080
)

$ErrorActionPreference = "Stop"

$repoRoot = Resolve-Path (Join-Path $PSScriptRoot "..")
Set-Location $repoRoot

# Avoid inheriting broken local proxy settings (for example 127.0.0.1:9).
$env:HTTP_PROXY = ""
$env:HTTPS_PROXY = ""
$env:ALL_PROXY = ""
$env:http_proxy = ""
$env:https_proxy = ""
$env:all_proxy = ""
$env:GIT_HTTP_PROXY = ""
$env:GIT_HTTPS_PROXY = ""

$python = if ($env:OBS_HF_PYTHON -and $env:OBS_HF_PYTHON.Trim()) {
  $env:OBS_HF_PYTHON
} else {
  "python"
}

Write-Host "OBSTRAL Lite UI: http://$ListenHost`:$Port/"
& $python ".\scripts\serve_lite.py" --host $ListenHost --port $Port
