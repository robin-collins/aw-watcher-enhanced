# Load a Model (`POST /api/v1/models/load`)

[README](README.md) · Prev: [List Models](list-models.md) · Next: [Unload Model](unload-model.md)

Loads a model into memory with optional configuration.

## Linux (curl)
```bash
curl http://localhost:1234/api/v1/models/load   -H "Content-Type: application/json"   -H "Authorization: Bearer $LM_API_TOKEN"   -d '{
    "model": "openai/gpt-oss-20b",
    "context_length": 16384,
    "flash_attention": true,
    "echo_load_config": true
  }'
```

## Windows (PowerShell)
```powershell
$uri = "http://localhost:1234/api/v1/models/load"
$headers = @{"Content-Type"="application/json"}
# $headers["Authorization"] = "Bearer $env:LM_API_TOKEN"  # optional
$body = @{
  model = "openai/gpt-oss-20b"
  context_length = 16384
  flash_attention = $true
  echo_load_config = $true
} | ConvertTo-Json

Invoke-WebRequest -Uri $uri -Method POST -Headers $headers -Body $body |
  Select-Object -ExpandProperty Content
```

