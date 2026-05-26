# List Models (`GET /api/v1/models`)

[README](README.md) · Prev: [Streaming Events](streaming-events.md) · Next: [Load Model](load-model.md)

Returns available models on the system, including loaded instances and capabilities.

## Linux (curl)
```bash
curl http://localhost:1234/api/v1/models   -H "Authorization: Bearer $LM_API_TOKEN"
```

## Windows (PowerShell)
```powershell
Invoke-WebRequest -Uri "http://localhost:1234/api/v1/models" -Method GET -Headers @{
  Authorization = "Bearer $env:LM_API_TOKEN"  # optional
} | Select-Object -ExpandProperty Content
```

Tip: look for `capabilities.vision` to find vision models.

