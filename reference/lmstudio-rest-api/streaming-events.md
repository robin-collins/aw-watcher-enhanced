# Streaming Events (SSE)

[README](README.md) · Prev: [Stateful Chats](stateful-chats.md) · Next: [List Models](list-models.md)

When you set `"stream": true` in `/api/v1/chat`, LM Studio emits **Server‑Sent Events** (SSE) with event names such as:

- `chat.start` … `chat.end`
- `model_load.*` (if the model needs loading)
- `prompt_processing.*`
- `reasoning.*`
- `message.*`
- `tool_call.*`
- `error`

## Linux: stream with curl

```bash
curl -N http://localhost:1234/api/v1/chat   -H "Content-Type: application/json"   -H "Authorization: Bearer $LM_API_TOKEN"   -d '{
    "model": "ibm/granite-4-micro",
    "input": "Explain VLANs like I"m new to networking.",
    "stream": true
  }'
```

> `-N` disables curl buffering so you see events as they arrive.

## Windows: stream with PowerShell

PowerShell doesn’t have first‑class SSE parsing built‑in, but you can still read the event stream as text.

```powershell
$uri = "http://localhost:1234/api/v1/chat"
$body = @{ model = "ibm/granite-4-micro"; input = "Explain VLANs like I'm new to networking."; stream = $true } | ConvertTo-Json

$req = [System.Net.HttpWebRequest]::Create($uri)
$req.Method = "POST"
$req.ContentType = "application/json"
# $req.Headers.Add("Authorization", "Bearer $env:LM_API_TOKEN")  # optional

$bytes = [System.Text.Encoding]::UTF8.GetBytes($body)
$req.ContentLength = $bytes.Length
$stream = $req.GetRequestStream()
$stream.Write($bytes, 0, $bytes.Length)
$stream.Close()

$resp = $req.GetResponse()
$reader = New-Object System.IO.StreamReader($resp.GetResponseStream())
while(-not $reader.EndOfStream){
  $line = $reader.ReadLine()
  if($line){ $line }
}
$reader.Close(); $resp.Close()
```

