Param(
  [Parameter(ValueFromRemainingArguments = $true)]
  [string[]]$Args
)

$ErrorActionPreference = "Stop"

$repoRoot = Resolve-Path (Join-Path $PSScriptRoot "..")
Set-Location $repoRoot

$python = if ($env:OBS_HF_PYTHON -and $env:OBS_HF_PYTHON.Trim()) {
  $env:OBS_HF_PYTHON
} else {
  "python"
}

& $python ".\scripts\obstral_lite_cli.py" @Args
