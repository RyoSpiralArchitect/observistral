Param(
  [string]$Remote = "origin",
  [string]$Branch = "main",
  [string]$Token = "",
  [string]$Username = "x-access-token"
)

$ErrorActionPreference = "Stop"

if (-not $Token) {
  $Token = $env:GITHUB_TOKEN
}

if (-not $Token) {
  Write-Host "[push] Missing token. Set env:GITHUB_TOKEN or pass -Token." -ForegroundColor Red
  exit 2
}

# Some locked-down environments poison proxy env vars (e.g. http://127.0.0.1:9).
$env:HTTP_PROXY = ""
$env:HTTPS_PROXY = ""
$env:ALL_PROXY = ""
$env:GIT_HTTP_PROXY = ""
$env:GIT_HTTPS_PROXY = ""

# Avoid any interactive prompts (which may trigger msys sh.exe and fail under WDAC).
$env:GIT_TERMINAL_PROMPT = "0"

# GitHub accepts PAT via HTTP Basic. Use an extra header so we don't rewrite remotes.
$pair = "$Username`:$Token"
$b64 = [Convert]::ToBase64String([Text.Encoding]::ASCII.GetBytes($pair))
$hdr = "AUTHORIZATION: basic $b64"

Write-Host ("[push] git push {0} {1}" -f $Remote, $Branch) -ForegroundColor Cyan
git -c http.sslBackend=openssl -c ("http.extraheader=" + $hdr) push $Remote $Branch

