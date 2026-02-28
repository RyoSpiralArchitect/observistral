Param(
  [string]$Host = "127.0.0.1",
  [int]$Port = 18080
)

$ErrorActionPreference = "Stop"

$repoRoot = Resolve-Path (Join-Path $PSScriptRoot "..")
Set-Location $repoRoot

$cargoBin = Join-Path $env:USERPROFILE ".cargo\\bin"
$cargoExe = Join-Path $cargoBin "cargo.exe"
$exe = Join-Path $repoRoot "target\\debug\\obstral.exe"

if (Test-Path $cargoExe) {
  & $cargoExe build
  if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
}

if (-not (Test-Path $exe)) {
  Write-Host "obstral.exe not found at: $exe"
  Write-Host "Run: .\\scripts\\install.ps1   (or install Rust and build with cargo)"
  exit 1
}

Write-Host "OBSTRAL UI: http://$Host`:$Port/"
& $exe serve --host $Host --port $Port
