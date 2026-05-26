# Image Input (base64 data URLs)

[README](README.md) · Prev: [MCP via API](mcp-via-api.md) · Next: [Utilities: Base64 helpers](utilities-base64.md)

Vision‑language models (VLMs) can accept images. In `/api/v1/chat`, provide `input` as an array of typed objects.

- Text: `{ "type": "message", "content": "..." }`
- Image: `{ "type": "image", "data_url": "data:image/png;base64,..." }`

## 1) Convert an image to base64

See [Utilities: Base64 helpers](utilities-base64.md) for ready-to-use snippets.

## 2) Send image + text (Linux curl)

```bash
IMG_B64=$(base64 -w 0 ./image.png)
DATA_URL="data:image/png;base64,$IMG_B64"

curl http://localhost:1234/api/v1/chat   -H "Content-Type: application/json"   -H "Authorization: Bearer $LM_API_TOKEN"   -d "$(jq -n --arg url "$DATA_URL" '{
    model: "qwen2-vl-2b-instruct",
    input: [
      {type: "message", content: "Describe this image."},
      {type: "image", data_url: $url}
    ]
  }')"
```

> If you don’t have `jq`, you can inline JSON manually — just be careful with quoting.

## 3) Send image + text (Windows PowerShell)

```powershell
$bytes = [System.IO.File]::ReadAllBytes(".\image.png")
$b64 = [System.Convert]::ToBase64String($bytes)
$dataUrl = "data:image/png;base64,$b64"

$body = @{
  model = "qwen2-vl-2b-instruct"
  input = @(
    @{ type = "message"; content = "Describe this image." },
    @{ type = "image"; data_url = $dataUrl }
  )
} | ConvertTo-Json -Depth 8

Invoke-WebRequest -Uri "http://localhost:1234/api/v1/chat" -Method POST -Headers @{
  "Content-Type" = "application/json"
  # "Authorization" = "Bearer $env:LM_API_TOKEN"  # optional
} -Body $body | Select-Object -ExpandProperty Content
```

