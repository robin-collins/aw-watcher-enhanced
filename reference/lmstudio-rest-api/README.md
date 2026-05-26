# LM Studio REST API (v1) — Practical Wiki

This mini-wiki is built from LM Studio’s official REST API docs and focuses on **copy‑pasteable** examples for:

- Linux **bash** + `curl`
- Windows **PowerShell** + `Invoke-WebRequest`
- Variations including **streaming (SSE)**, **stateful chats**, **MCP integrations**, and **image input** (base64)

> Default server URL: `http://localhost:1234`.

## Index

1. [REST API Overview](rest-api-overview.md)
2. [Quickstart](quickstart.md)
3. [Endpoints & Versions (v0 vs v1)](endpoints-and-versions.md)
4. [Chat (`POST /api/v1/chat`)](chat.md)
5. [Stateful Chats](stateful-chats.md)
6. [Streaming Events (SSE)](streaming-events.md)
7. [List Models (`GET /api/v1/models`)](list-models.md)
8. [Load Model (`POST /api/v1/models/load`)](load-model.md)
9. [Unload Model (`POST /api/v1/models/unload`)](unload-model.md)
10. [Download Model (`POST /api/v1/models/download`)](download-model.md)
11. [Download Status (`GET /api/v1/models/download/status/:job_id`)](download-status.md)
12. [MCP via API](mcp-via-api.md)
13. [Image Input (base64 data URLs)](image-input.md)
14. [Utilities: Base64 helpers](utilities-base64.md)

## Conventions

- Examples assume LM Studio is running with the server enabled.
- If you have enabled token auth, set:
  - Linux: `export LM_API_TOKEN=...`
  - Windows: `$env:LM_API_TOKEN = "..."`

---

Generated: 2026-04-08T07:39:36
