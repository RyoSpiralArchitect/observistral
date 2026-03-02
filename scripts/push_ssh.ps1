Param(
  [string]$Remote = "origin",
  [string]$Branch = "main",
  [string]$HostName = "ssh.github.com",
  [int]$Port = 443,
  [string]$IdentityFile = ""
)

$ErrorActionPreference = "Stop"

if (-not $IdentityFile) {
  $IdentityFile = Join-Path $env:USERPROFILE ".ssh\\id_ed25519"
}

if (-not (Test-Path $IdentityFile)) {
  Write-Host ("[push_ssh] Missing IdentityFile: {0}" -f $IdentityFile) -ForegroundColor Red
  Write-Host "[push_ssh] Generate one with: C:\\Windows\\System32\\OpenSSH\\ssh-keygen.exe -t ed25519 -f %USERPROFILE%\\.ssh\\id_ed25519" -ForegroundColor Yellow
  exit 2
}

# Some locked-down environments poison proxy env vars (e.g. http://127.0.0.1:9).
$env:HTTP_PROXY = ""
$env:HTTPS_PROXY = ""
$env:ALL_PROXY = ""
$env:GIT_HTTP_PROXY = ""
$env:GIT_HTTPS_PROXY = ""

# Avoid any interactive prompts.
$env:GIT_TERMINAL_PROMPT = "0"

# Force a known-good ssh.exe (avoid PATH shims / WDAC oddities) and ignore user ssh config
# because OpenSSH on Windows is strict about ACLs on ~/.ssh/config.
$sshExe = "C:/Windows/System32/OpenSSH/ssh.exe"
$keyPosix = ($IdentityFile -replace "\\\\", "/")
$env:GIT_SSH_COMMAND = "$sshExe -F NUL -o BatchMode=yes -o StrictHostKeyChecking=accept-new -o HostName=$HostName -p $Port -i `"$keyPosix`" -o IdentitiesOnly=yes"

Write-Host ("[push_ssh] git push {0} {1} (via {2}:{3})" -f $Remote, $Branch, $HostName, $Port) -ForegroundColor Cyan
git push $Remote $Branch

