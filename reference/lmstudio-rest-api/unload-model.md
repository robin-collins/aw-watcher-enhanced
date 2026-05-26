# Unload a Model (`POST /api/v1/models/unload`)

[README](README.md) · Prev: [Load Model](load-model.md) · Next: [Download Model](download-model.md)

Unloads a previously loaded model instance.

## Linux (curl)
```bash
curl http://localhost:1234/api/v1/models/unload   -H "Content-Type: application/json"   -H "Authorization: Bearer $LM_API_TOKEN"   -d '{
    "instance_id": "openai/gpt-oss-20b"
  }'
```

## Windows (PowerShell)
```powershell
Invoke-WebRequest -Uri "http://localhost:1234/api/v1/models/unload" -Method POST -Headers @{
  "Content-Type" = "application/json"
  # "Authorization" = "Bearer $env:LM_API_TOKEN"  # optional
} -Body (@{ instance_id = "openai/gpt-oss-20b" } | ConvertTo-Json) |
  Select-Object -ExpandProperty Content
```

