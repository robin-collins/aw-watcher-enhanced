# Endpoints & Versions (v0 vs v1)

[README](README.md) · Prev: [Quickstart](quickstart.md) · Next: [Chat](chat.md)

LM Studio has:

- **Native v1** API: `/api/v1/*` (recommended; supports stateful chats, streaming events, MCP)
- **Native v0** API: `/api/v0/*` (older; richer stats in some responses)
- **OpenAI-compatible** API: `/v1/*` (drop-in for OpenAI clients)

## Native v1 endpoints (common)

- `POST /api/v1/chat`
- `GET  /api/v1/models`
- `POST /api/v1/models/load`
- `POST /api/v1/models/unload`
- `POST /api/v1/models/download`
- `GET  /api/v1/models/download/status/:job_id`

## Native v0 (legacy)

If you still need v0, see LM Studio “REST API v0” docs and use:

- `GET /api/v0/models`
- `POST /api/v0/chat/completions`

### Example: list v0 models (curl)
```bash
curl http://localhost:1234/api/v0/models   -H "Authorization: Bearer $LM_API_TOKEN"
```

### Example: list v0 models (PowerShell)
```powershell
Invoke-WebRequest -Uri "http://localhost:1234/api/v0/models" -Method GET -Headers @{
  Authorization = "Bearer $env:LM_API_TOKEN"  # optional
} | Select-Object -ExpandProperty Content
```

