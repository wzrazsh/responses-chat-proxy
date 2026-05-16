use axum::{
    response::IntoResponse,
    Json,
};

pub async fn test_page_handler() -> impl IntoResponse {
    Html(TEST_PAGE_HTML)
}

pub async fn api_status_handler() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "running",
        "version": "0.1.0"
    }))
}

pub struct Html<T>(pub T);

impl<T: AsRef<str>> IntoResponse for Html<T> {
    fn into_response(self) -> axum::response::Response {
        axum::response::Html(self.0.as_ref().to_string()).into_response()
    }
}

const TEST_PAGE_HTML: &str = r#"
<!DOCTYPE html>
<html lang="zh-CN">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Responses Proxy 测试</title>
    <style>
        * { margin: 0; padding: 0; box-sizing: border-box; }
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            background: #1a1a2e;
            color: #eee;
            min-height: 100vh;
            padding: 20px;
        }
        .container { max-width: 900px; margin: 0 auto; }
        h1 { text-align: center; margin-bottom: 30px; color: #00d4ff; }
        .panel { background: #16213e; border-radius: 12px; padding: 24px; margin-bottom: 20px; }
        .panel h2 { margin-bottom: 16px; color: #00d4ff; font-size: 1.1rem; }
        .form-group { margin-bottom: 16px; }
        label { display: block; margin-bottom: 6px; color: #aaa; font-size: 0.9rem; }
        select, textarea { width: 100%; padding: 12px; border: 1px solid #333; border-radius: 8px; background: #0f3460; color: #fff; font-size: 1rem; outline: none; }
        select:focus, textarea:focus { border-color: #00d4ff; }
        textarea { min-height: 120px; resize: vertical; }
        .btn { padding: 12px 24px; border: none; border-radius: 8px; font-size: 1rem; cursor: pointer; transition: all 0.2s; }
        .btn-primary { background: #00d4ff; color: #1a1a2e; }
        .btn-primary:hover { background: #00a8cc; }
        .btn-primary:disabled { background: #555; cursor: not-allowed; }
        .response-area { background: #0f3460; border-radius: 8px; padding: 16px; min-height: 200px; white-space: pre-wrap; font-family: monospace; overflow-x: auto; }
        .status { display: flex; gap: 20px; flex-wrap: wrap; }
        .status-item { background: #0f3460; padding: 12px 16px; border-radius: 8px; }
        .status-item .label { color: #888; font-size: 0.85rem; }
        .status-item .value { color: #00d4ff; font-size: 1.2rem; font-weight: bold; }
        .loading { display: none; color: #00d4ff; text-align: center; padding: 20px; }
        .error { color: #ff6b6b; white-space: pre-wrap; }
    </style>
</head>
<body>
    <div class="container">
        <h1>🤖 Responses Proxy 测试页面</h1>

        <div class="panel">
            <h2>服务状态</h2>
            <div class="status" id="status">
                <div class="status-item">
                    <div class="label">状态</div>
                    <div class="value" id="service-status">检测中...</div>
                </div>
                <div class="status-item">
                    <div class="label">版本</div>
                    <div class="value" id="version">-</div>
                </div>
            </div>
        </div>

        <div class="panel">
            <h2>发送测试请求</h2>
            <div class="form-group">
                <label for="model">模型</label>
                <select id="model">
                    <option value="opencode-deepseek-v4-flash">opencode-deepseek-v4-flash</option>
                    <option value="opencode-gpt-4o">opencode-gpt-4o</option>
                    <option value="opencode-claude-sonnet-4">opencode-claude-sonnet-4</option>
                    <option value="opencode-qwen3.6-plus">opencode-qwen3.6-plus</option>
                    <option value="deepseek-v4-flash">deepseek-v4-flash</option>
                    <option value="deepseek-chat">deepseek-chat</option>
                    <option value="deepseek-reasoner">deepseek-reasoner</option>
                </select>
            </div>
            <div class="form-group">
                <label for="input">输入内容</label>
                <textarea id="input" placeholder="输入你的问题...">Hello, are you working?</textarea>
            </div>
            <button class="btn btn-primary" id="sendBtn" onclick="sendRequest()">发送请求</button>
        </div>

        <div class="panel">
            <h2>响应结果</h2>
            <div class="loading" id="loading">正在等待响应...</div>
            <div class="response-area" id="response">响应内容将显示在这里...</div>
        </div>
    </div>

    <script>
        async function checkStatus() {
            try {
                const resp = await fetch('/health');
                const data = await resp.json();
                document.getElementById('service-status').textContent = data.ok ? '✅ 运行中' : '❌ 异常';

                const statusResp = await fetch('/api/status');
                if (statusResp.ok) {
                    const status = await statusResp.json();
                    document.getElementById('version').textContent = status.version;
                }
            } catch (e) {
                document.getElementById('service-status').textContent = '❌ 连接失败';
            }
        }

        async function sendRequest() {
            const btn = document.getElementById('sendBtn');
            const loading = document.getElementById('loading');
            const response = document.getElementById('response');

            btn.disabled = true;
            loading.style.display = 'block';
            response.textContent = '';

            const model = document.getElementById('model').value;
            const input = document.getElementById('input').value;

            try {
                const resp = await fetch('/v1/responses', {
                    method: 'POST',
                    headers: {
                        'Content-Type': 'application/json',
                        'Authorization': 'Bearer dummy'
                    },
                    body: JSON.stringify({
                        model: model,
                        input: input,
                        max_output_tokens: 500
                    })
                });

                const data = await resp.json();
                loading.style.display = 'none';

                if (resp.ok) {
                    response.textContent = JSON.stringify(data, null, 2);
                } else {
                    response.className = 'response-area error';
                    response.textContent = '错误 (' + resp.status + '): ' + JSON.stringify(data, null, 2);
                }
            } catch (e) {
                loading.style.display = 'none';
                response.className = 'response-area error';
                response.textContent = '请求失败: ' + e.message;
            }

            btn.disabled = false;
        }

        checkStatus();
        setInterval(checkStatus, 5000);
    </script>
</body>
</html>
"#;