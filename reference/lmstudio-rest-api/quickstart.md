# Quickstart

[README](README.md) · Prev: [REST API Overview](rest-api-overview.md) · Next: [Endpoints & Versions](endpoints-and-versions.md)

This quickstart shows a minimal flow:

1. Start the server (`lms server start` or via LM Studio UI)
2. (Optional) download a small model
3. Send a chat request
4. (Optional) use MCP integrations
5. (Optional) download via API + check download status

## 1) Start the server

### Linux
```bash
lms server start
```

### Windows
```powershell
lms server start
```

## 2) Download a model (CLI)

```bash
lms get ibm/granite-4-micro
```

## 3) Chat with a model (REST)

### Linux (curl)
```bash
curl http://localhost:1234/api/v1/chat   -H "Content-Type: application/json"   -H "Authorization: Bearer $LM_API_TOKEN"   -d '{
    "model": "ibm/granite-4-micro",
    "input": "Write a short haiku about sunrise."
  }'
```

### Windows (PowerShell)
```powershell
$uri = "http://localhost:1234/api/v1/chat"
$body = @{ model = "ibm/granite-4-micro"; input = "Write a short haiku about sunrise." } | ConvertTo-Json
$headers = @{"Content-Type"="application/json"}
# $headers["Authorization"] = "Bearer $env:LM_API_TOKEN"  # optional

Invoke-WebRequest -Uri $uri -Method POST -Headers $headers -Body $body |
  Select-Object -ExpandProperty Content
```

## 4) MCP servers via API (ephemeral + plugin)

See full details in [MCP via API](mcp-via-api.md).

## 5) Download via API + status

- Start download: [Download Model](download-model.md)
- Poll status: [Download Status](download-status.md)

