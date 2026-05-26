# Utilities: Base64 helpers

[README](README.md) · Prev: [Image Input](image-input.md) · Next: (none)

These helper snippets convert local images to **base64** and format them as **data URLs** suitable for `data_url`.

## Linux bash

```bash
# Linux: convert image to a single-line base64 (no wraps)
# PNG example
IMG_B64=$(base64 -w 0 ./image.png)
DATA_URL="data:image/png;base64,$IMG_B64"

# JPEG example
IMG_B64=$(base64 -w 0 ./photo.jpg)
DATA_URL="data:image/jpeg;base64,$IMG_B64"

# macOS note: use `base64 -b 0` instead of `-w 0`

```

## Windows PowerShell

```powershell
# Windows PowerShell: convert image to base64 and build a data URL
$bytes = [System.IO.File]::ReadAllBytes(".\image.png")
$b64   = [System.Convert]::ToBase64String($bytes)
$dataUrl = "data:image/png;base64,$b64"

# JPEG example
$bytes = [System.IO.File]::ReadAllBytes(".\photo.jpg")
$b64   = [System.Convert]::ToBase64String($bytes)
$dataUrl = "data:image/jpeg;base64,$b64"

```

## Common JSON fragment

```json
{
  "type": "image",
  "data_url": "data:image/png;base64,...."
}
```

