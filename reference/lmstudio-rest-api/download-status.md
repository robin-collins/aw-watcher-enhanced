# Download Status (`GET /api/v1/models/download/status/:job_id`)

[README](README.md) · Prev: [Download Model](download-model.md) · Next: [MCP via API](mcp-via-api.md)

Poll a download job until it completes.

## Linux (curl)
```bash
curl http://localhost:1234/api/v1/models/download/status/job_493c7c9ded   -H "Authorization: Bearer $LM_API_TOKEN"
```

## Windows (PowerShell)
```powershell
$job = "job_493c7c9ded"
Invoke-WebRequest -Uri "http://localhost:1234/api/v1/models/download/status/$job" -Method GET -Headers @{
  Authorization = "Bearer $env:LM_API_TOKEN"  # optional
} | Select-Object -ExpandProperty Content
```

