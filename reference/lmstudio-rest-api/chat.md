# Chat (`POST /api/v1/chat`)

[README](README.md) · Prev: [Endpoints & Versions](endpoints-and-versions.md) · Next: [Stateful Chats](stateful-chats.md)

The `/api/v1/chat` endpoint sends input to a model and returns an array of output items.

## Minimal request (text)

### Linux (curl)
```bash
curl http://localhost:1234/api/v1/chat   -H "Content-Type: application/json"   -H "Authorization: Bearer $LM_API_TOKEN"   -d '{
    "model": "ibm/granite-4-micro",
    "input": "Summarise what DHCP does in one paragraph."
  }'
```

### Windows (PowerShell)
```powershell
$uri = "http://localhost:1234/api/v1/chat"
$headers = @{"Content-Type"="application/json"}
# $headers["Authorization"] = "Bearer $env:LM_API_TOKEN"  # optional
$body = @{ model = "ibm/granite-4-micro"; input = "Summarise what DHCP does in one paragraph." } | ConvertTo-Json
Invoke-WebRequest -Uri $uri -Method POST -Headers $headers -Body $body |
  Select-Object -ExpandProperty Content
```

## Request with MCP integrations

See [MCP via API](mcp-via-api.md) for full patterns.

## Request with streaming

Set `"stream": true` and consume Server‑Sent Events (SSE). See [Streaming Events](streaming-events.md).

## Request with images

Use `input` as an array of objects with `type: "image"` and `data_url: "data:image/...;base64,..."`. See [Image Input](image-input.md).

