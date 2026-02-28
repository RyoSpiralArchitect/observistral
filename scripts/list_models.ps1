$r = Invoke-RestMethod -Uri 'https://api.mistral.ai/v1/models' -Headers @{Authorization="Bearer $env:MISTRAL_API_KEY"}
$r.data | Where-Object { $_.id -match 'dev|code|stral' } | Select-Object -ExpandProperty id | Sort-Object
