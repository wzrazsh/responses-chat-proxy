# Responses Chat Proxy

Local HTTP proxy that exposes an OpenAI Responses-compatible API and converts requests to Chat Completions upstream. Lets newer Codex versions use providers like DeepSeek and MiniMax that only support `/chat/completions`.

## Quick Start

```powershell
$env:DEEPSEEK_API_KEY = "sk-..."
cargo run
```

```toml
# codex config
[model_providers.deepseek_proxy]
name = "Local DeepSeek Responses Proxy"
base_url = "http://127.0.0.1:8787/v1"
env_key = "DEEPSEEK_API_KEY"
wire_api = "responses"
requires_openai_auth = false
```

```powershell
codex exec -p deepseek_proxy "Reply exactly CODEX_PROXY_OK"
```

## API

### `GET /health`

```json
{ "ok": true, "service": "responses-chat-proxy" }
```

### `POST /v1/responses`

String input:
```json
{
  "model": "deepseek-chat",
  "input": "hello",
  "instructions": "system prompt",
  "max_output_tokens": 1024,
  "stream": false
}
```

Array input:
```json
{
  "model": "deepseek-chat",
  "input": [{ "role": "user", "content": [{ "type": "input_text", "text": "hello" }] }],
  "instructions": "system prompt"
}
```

## Environment

| Variable | Default | Description |
|---|---|---|
| `PROXY_BIND_ADDR` | `127.0.0.1:8787` | Listen address |
| `DEEPSEEK_API_KEY` | — | DeepSeek API key |
| `MINIMAX_API_KEY` | — | MiniMax API key |
| `DEFAULT_PROVIDER` | `deepseek` | Fallback provider |
| `UPSTREAM_TIMEOUT_SECS` | `300` | Upstream request timeout |
| `PROXY_LOG_PROMPTS` | — | Set to `1` to log full prompts |

## Model Routing

- `deepseek-*` → DeepSeek
- `MiniMax-*`, `minimax-*`, `codex-MiniMax-*` → MiniMax
- others → `DEFAULT_PROVIDER`

## Project Layout

```
src/
├── main.rs       — Axum server, routes, handlers
├── config.rs     — Env config and provider definitions
├── error.rs      — AppError → JSON error responses
├── providers.rs  — Model-based provider routing
├── responses.rs  — Responses API types
├── chat.rs       — Request/response conversion
└── stream.rs     — SSE streaming (Chat delta → Responses event)
```

## License

MIT
