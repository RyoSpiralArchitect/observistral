param(
  [switch]$Force = $true
)

$procs = Get-Process -Name obstral -ErrorAction SilentlyContinue
if (-not $procs) {
  Write-Host "[kill-obstral] no obstral process"
  exit 0
}

Write-Host ("[kill-obstral] stopping {0} process(es)..." -f $procs.Count)
if ($Force) {
  $procs | Stop-Process -Force
} else {
  $procs | Stop-Process
}
Write-Host "[kill-obstral] done"

