# Download a Model (`POST /api/v1/models/download`)

[README](README.md) · Prev: [Unload Model](unload-model.md) · Next: [Download Status](download-status.md)

Downloads models by model-catalog identifier or exact Hugging Face URL.

## Download by catalogue identifier

### Linux (curl)
```bash
curl http://localhost:1234/api/v1/models/download   -H "Content-Type: application/json"   -H "Authorization: Bearer $LM_API_TOKEN"   -d '{
    "model": "ibm/granite-4-micro"
  }'
```

### Windows (PowerShell)
```powershell
Invoke-WebRequest -Uri "http://localhost:1234/api/v1/models/download" -Method POST -Headers @{
  "Content-Type" = "application/json"
  # "Authorization" = "Bearer $env:LM_API_TOKEN"  # optional
} -Body (@{ model = "ibm/granite-4-micro" } | ConvertTo-Json) |
  Select-Object -ExpandProperty Content
```

## Download by Hugging Face URL + quantisation

### Linux (curl)
```bash
curl http://localhost:1234/api/v1/models/download   -H "Content-Type: application/json"   -H "Authorization: Bearer $LM_API_TOKEN"   -d '{
    "model": "https://huggingface.co/lmstudio-community/gpt-oss-20b-GGUF",
    "quantization": "Q4_K_M"
  }'
```

### Windows (PowerShell)
```powershell
$body = @{ 
  model = "https://huggingface.co/lmstudio-community/gpt-oss-20b-GGUF"
  quantization = "Q4_K_M"
} | ConvertTo-Json
Invoke-WebRequest -Uri "http://localhost:1234/api/v1/models/download" -Method POST -Headers @{
  "Content-Type" = "application/json"
  # "Authorization" = "Bearer $env:LM_API_TOKEN"  # optional
} -Body $body | Select-Object -ExpandProperty Content
```

The response includes a `job_id` unless the model is already downloaded.

