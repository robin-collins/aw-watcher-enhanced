# Stateful Chats

[README](README.md) · Prev: [Chat](chat.md) · Next: [Streaming Events](streaming-events.md)

`/api/v1/chat` is **stateful by default**. The server returns a `response_id` you can pass back as `previous_response_id` to continue the thread.

## Start a new conversation

### Linux (curl)
```bash
curl http://localhost:1234/api/v1/chat   -H "Content-Type: application/json"   -H "Authorization: Bearer $LM_API_TOKEN"   -d '{
    "model": "ibm/granite-4-micro",
    "input": "My favourite colour is blue."
  }'
```

### Windows (PowerShell)
```powershell
$resp = Invoke-WebRequest -Uri "http://localhost:1234/api/v1/chat" -Method POST -Headers @{
  "Content-Type" = "application/json"
  # "Authorization" = "Bearer $env:LM_API_TOKEN"  # optional
} -Body (@{ model = "ibm/granite-4-micro"; input = "My favourite colour is blue." } | ConvertTo-Json)

$resp.Content
```

## Continue the conversation

Take the `response_id` from the prior response.

### Linux (curl)
```bash
curl http://localhost:1234/api/v1/chat   -H "Content-Type: application/json"   -H "Authorization: Bearer $LM_API_TOKEN"   -d '{
    "model": "ibm/granite-4-micro",
    "input": "What colour did I just mention?",
    "previous_response_id": "resp_abc123..."
  }'
```

### Windows (PowerShell)
```powershell
$prev = "resp_abc123..."
Invoke-WebRequest -Uri "http://localhost:1234/api/v1/chat" -Method POST -Headers @{
  "Content-Type" = "application/json"
  # "Authorization" = "Bearer $env:LM_API_TOKEN"  # optional
} -Body (@{ model = "ibm/granite-4-micro"; input = "What colour did I just mention?"; previous_response_id = $prev } | ConvertTo-Json) |
  Select-Object -ExpandProperty Content
```

## Disable storage (stateless)

Set `"store": false`.

