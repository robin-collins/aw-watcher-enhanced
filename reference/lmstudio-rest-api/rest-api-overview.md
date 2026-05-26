# REST API Overview (v1)

[README](README.md) · Prev: (none) · Next: [Quickstart](quickstart.md)

LM Studio provides a **native v1 REST API** under `/api/v1/*` for local inference and model management, including:

- Chat: `POST /api/v1/chat`
- Model listing: `GET /api/v1/models`
- Model lifecycle: load/unload
- Model downloads + status
- Advanced features like **stateful chats** and **MCP via API**

## Base URL & Port

- Default: `http://localhost:1234`
- You can change the port in LM Studio settings.

## Authentication

- By default, the API **may be unauthenticated**, but you can enable **API tokens** in server settings.
- If enabled, add: `Authorization: Bearer <token>`

### Linux (curl) — health-ish check via model list

```bash
curl http://localhost:1234/api/v1/models   -H "Authorization: Bearer $LM_API_TOKEN"
```

### Windows (PowerShell) — model list

```powershell
$headers = @{}
$headers["Authorization"] = "Bearer $env:LM_API_TOKEN"  # optional
Invoke-WebRequest -Uri "http://localhost:1234/api/v1/models" -Headers $headers -Method GET |
  Select-Object -ExpandProperty Content
```

## Related

- [Endpoints & Versions](endpoints-and-versions.md)
- [Chat](chat.md)
- [MCP via API](mcp-via-api.md)

