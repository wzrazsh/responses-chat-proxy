# Responses Chat Proxy

Local HTTP proxy that exposes an OpenAI Responses-compatible API and converts requests to Chat Completions upstream. Lets newer Codex versions use providers like DeepSeek, MiniMax, and OpenCode Go that only support `/chat/completions`.

## Quick Start

### DeepSeek

```powershell
$env:DEEPSEEK_API_KEY = "sk-..."
cargo run
```

### MiniMax

```powershell
$env:MINIMAX_API_KEY = "sk-..."
cargo run
```

### OpenCode Go

OpenCode Go 认证支持三种方式（按优先级）：

1. **环境变量**（最高优先级）
   ```powershell
   $env:OPENCODE_API_KEY = "sk-..."
   cargo run
   ```

2. **OpenCode auth.json**（自动读取）
   
   如果你已登录 OpenCode CLI，proxy 会自动读取认证文件：
   - Windows: `C:\Users\<用户名>\.local\share\opencode\auth.json`
   - Linux/macOS: `~/.local/share/opencode/auth.json`

3. **OPENCODE_AUTH_CONTENT 环境变量**
   ```powershell
   $env:OPENCODE_AUTH_CONTENT = '{"opencode-go":{"type":"api","key":"sk-..."}}'
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

OpenCode Go 示例：
```json
{
  "model": "opencode-deepseek-v4-flash",
  "input": "hello",
  "max_output_tokens": 1024,
  "stream": false
}
```

## Environment

| Variable | Default | Description |
|---|---|---|
| `PROXY_BIND_ADDR` | `127.0.0.1:8787` | Listen address |
| `DEEPSEEK_API_KEY` | — | DeepSeek API key |
| `MINIMAX_API_KEY` | — | MiniMax API key |
| `OPENCODE_API_KEY` | — | OpenCode Go API key |
| `OPENCODE_AUTH_CONTENT` | — | OpenCode auth JSON string |
| `DEFAULT_PROVIDER` | `deepseek` | Fallback provider |
| `UPSTREAM_TIMEOUT_SECS` | `300` | Upstream request timeout |
| `PROXY_LOG_PROMPTS` | — | Set to `1` to log full prompts |

## Model Routing

| 模型前缀 | 提供商 | 上游端点 |
|---|---|---|
| `deepseek-*` | DeepSeek | `https://api.deepseek.com/chat/completions` |
| `MiniMax-*`, `minimax-*`, `codex-MiniMax-*` | MiniMax | `https://api.minimaxi.com/v1/chat/completions` |
| `opencode-*` | OpenCode Go | `https://opencode.ai/zen/go/v1/chat/completions` |
| others | `DEFAULT_PROVIDER` | — |

### OpenCode 模型名映射

Proxy 会自动剥离 `opencode-` 前缀，转换为上游真实模型名：

| Proxy 模型名 | 上游模型名 |
|---|---|
| `opencode-deepseek-v4-flash` | `deepseek-v4-flash` |
| `opencode-gpt-4o` | `gpt-4o` |
| `opencode-claude-sonnet-4` | `claude-sonnet-4` |

## 支持的 OpenCode Go 模型

通过 `GET /v1/models` 可获取完整列表，包括：

- `opencode-deepseek-v4-flash`
- `opencode-deepseek-v4-pro`
- `opencode-gpt-4o`
- `opencode-claude-sonnet-4`
- `opencode-kimi-k2.5`
- `opencode-kimi-k2.6`
- `opencode-glm-5`
- `opencode-glm-5.1`
- `opencode-qwen3.5-plus`
- `opencode-qwen3.6-plus`
- `opencode-minimax-m2.5`
- `opencode-minimax-m2.7`

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
