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

# Force a known-good ssh.exe (avoid PATH shims / WDAC oddities).
# Note: Do NOT use GIT_SSH_COMMAND here. Git for Windows may invoke MSYS sh.exe to interpret it,
# which can fail in locked-down environments (Win32 error 5). Prefer GIT_SSH + an explicit
# ssh:// URL that encodes host/port.
$sshExe = "C:/Windows/System32/OpenSSH/ssh.exe"
$env:GIT_SSH = $sshExe
$env:GIT_SSH_VARIANT = "ssh"

# Best-effort: ensure the host key is accepted without prompting.
try {
  & $sshExe -o BatchMode=yes -o StrictHostKeyChecking=accept-new -p $Port -T ("git@{0}" -f $HostName) | Out-Null
} catch {
  # Ignore: GitHub returns non-zero for no-shell access, and some environments block this probe.
}

$remoteUrl = (& git remote get-url $Remote).Trim()
if (-not $remoteUrl) {
  Write-Host ("[push_ssh] Missing remote URL for: {0}" -f $Remote) -ForegroundColor Red
  exit 3
}

# Convert common GitHub URL forms to an ssh:// URL pinned to ssh.github.com:443.
$path = $null
if ($remoteUrl -match '^git@github\.com:(.+)$') {
  $path = $Matches[1]
} elseif ($remoteUrl -match '^ssh://git@github\.com/?(.+)$') {
  $path = $Matches[1]
} elseif ($remoteUrl -match '^https://github\.com/(.+)$') {
  $path = $Matches[1]
} elseif ($remoteUrl -match '^ssh://git@ssh\.github\.com:\d+/?(.+)$') {
  $path = $Matches[1]
}

if (-not $path) {
  Write-Host ("[push_ssh] Unsupported remote URL (expected GitHub): {0}" -f $remoteUrl) -ForegroundColor Red
  exit 4
}

if (-not $path.EndsWith(".git")) {
  $path = $path + ".git"
}

if ($IdentityFile -ne (Join-Path $env:USERPROFILE ".ssh\\id_ed25519")) {
  Write-Host ("[push_ssh] Note: IdentityFile param is not passed to git/ssh in this mode. Ensure your key is loaded in ssh-agent or is the default key: {0}" -f (Join-Path $env:USERPROFILE ".ssh\\id_ed25519")) -ForegroundColor Yellow
}

$pushUrl = "ssh://git@{0}:{1}/{2}" -f $HostName, $Port, $path
Write-Host ("[push_ssh] git push {0} {1} (via {2}:{3})" -f $Remote, $Branch, $HostName, $Port) -ForegroundColor Cyan
git push $pushUrl $Branch
