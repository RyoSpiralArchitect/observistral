param(
  [switch]$Force = $true,
  [string]$PathContains = ""
)

$procs = Get-Process -Name obstral -ErrorAction SilentlyContinue
$needle = ""
if ($PathContains) {
  try { $needle = (Resolve-Path $PathContains).Path } catch { $needle = $PathContains }
  $procs = $procs | Where-Object { $_.Path -and ($_.Path -like ("*" + $needle + "*")) }
}
if (-not $procs) {
  if ($needle) { Write-Host "[kill-obstral] no obstral process (filtered)" }
  else { Write-Host "[kill-obstral] no obstral process" }
  exit 0
}

Write-Host ("[kill-obstral] stopping {0} process(es)..." -f $procs.Count)
if ($Force) {
  $procs | Stop-Process -Force
} else {
  $procs | Stop-Process
}
Write-Host "[kill-obstral] done"
