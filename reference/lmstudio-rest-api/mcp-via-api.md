# MCP via API (Model Context Protocol)

[README](README.md) · Prev: [Download Status](download-status.md) · Next: [Image Input](image-input.md)

LM Studio supports **MCP** through `/api/v1/chat` by providing an `integrations` array.

Two common integration types:

- **Ephemeral MCP**: define a remote MCP server per request (`type: "ephemeral_mcp"`, `server_url`, etc.)
- **Plugin**: reference an MCP server configured in `mcp.json` (`type: "plugin"`, `id: "mcp/<name>"`)

## Ephemeral MCP example (Hugging Face MCP)

### Linux (curl)
```bash
curl http://localhost:1234/api/v1/chat   -H "Content-Type: application/json"   -H "Authorization: Bearer $LM_API_TOKEN"   -d '{
    "model": "ibm/granite-4-micro",
    "input": "What is the top trending model on hugging face?",
    "integrations": [
      {
        "type": "ephemeral_mcp",
        "server_label": "huggingface",
        "server_url": "https://huggingface.co/mcp",
        "allowed_tools": ["model_search"]
      }
    ],
    "context_length": 8000
  }'
```

### Windows (PowerShell)
```powershell
$body = @{ 
  model = "ibm/granite-4-micro"
  input = "What is the top trending model on hugging face?"
  integrations = @(
    @{ type = "ephemeral_mcp"; server_label = "huggingface"; server_url = "https://huggingface.co/mcp"; allowed_tools = @("model_search") }
  )
  context_length = 8000
} | ConvertTo-Json -Depth 8

Invoke-WebRequest -Uri "http://localhost:1234/api/v1/chat" -Method POST -Headers @{
  "Content-Type" = "application/json"
  # "Authorization" = "Bearer $env:LM_API_TOKEN"  # optional
} -Body $body | Select-Object -ExpandProperty Content
```

## Plugin MCP example (mcp.json installed server)

### Linux (curl)
```bash
curl http://localhost:1234/api/v1/chat   -H "Content-Type: application/json"   -H "Authorization: Bearer $LM_API_TOKEN"   -d '{
    "model": "ibm/granite-4-micro",
    "input": "Open lmstudio.ai",
    "integrations": [
      {
        "type": "plugin",
        "id": "mcp/playwright",
        "allowed_tools": ["browser_navigate"]
      }
    ],
    "context_length": 8000
  }'
```

### Windows (PowerShell)
```powershell
$body = @{
  model = "ibm/granite-4-micro"
  input = "Open lmstudio.ai"
  integrations = @(
    @{ type = "plugin"; id = "mcp/playwright"; allowed_tools = @("browser_navigate") }
  )
  context_length = 8000
} | ConvertTo-Json -Depth 8

Invoke-WebRequest -Uri "http://localhost:1234/api/v1/chat" -Method POST -Headers @{
  "Content-Type" = "application/json"
  Authorization = "Bearer $env:LM_API_TOKEN"   # commonly required for local plugins
} -Body $body | Select-Object -ExpandProperty Content
```

