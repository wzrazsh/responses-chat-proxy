use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct ProviderConfig {
    pub name: String,
    pub base_url: String,
    pub chat_path: String,
    pub api_key_env: String,
    pub default_model: String,
}

#[derive(Clone, Debug)]
pub struct AppConfig {
    pub bind_addr: String,
    pub default_provider: String,
    pub log_prompts: bool,
    pub upstream_timeout_secs: u64,
    pub providers: HashMap<String, ProviderConfig>,
}

impl AppConfig {
    pub fn from_env() -> Self {
        let bind_addr = std::env::var("PROXY_BIND_ADDR")
            .unwrap_or_else(|_| "127.0.0.1:8787".to_string());
        let default_provider = std::env::var("DEFAULT_PROVIDER")
            .unwrap_or_else(|_| "deepseek".to_string());
        let log_prompts = std::env::var("PROXY_LOG_PROMPTS")
            .map(|v| v == "1")
            .unwrap_or(false);
        let upstream_timeout_secs = std::env::var("UPSTREAM_TIMEOUT_SECS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(300);

        let mut providers = HashMap::new();
        providers.insert(
            "deepseek".to_string(),
            ProviderConfig {
                name: "DeepSeek".to_string(),
                base_url: "https://api.deepseek.com".to_string(),
                chat_path: "/chat/completions".to_string(),
                api_key_env: "DEEPSEEK_API_KEY".to_string(),
                default_model: "deepseek-chat".to_string(),
            },
        );
        providers.insert(
            "minimax".to_string(),
            ProviderConfig {
                name: "MiniMax".to_string(),
                base_url: "https://api.minimaxi.com/v1".to_string(),
                chat_path: "/chat/completions".to_string(),
                api_key_env: "MINIMAX_API_KEY".to_string(),
                default_model: "codex-MiniMax-M2.7".to_string(),
            },
        );
        providers.insert(
            "opencode".to_string(),
            ProviderConfig {
                name: "OpenCode".to_string(),
                base_url: "https://opencode.ai/zen/go/v1".to_string(),
                chat_path: "/chat/completions".to_string(),
                api_key_env: "OPENCODE_API_KEY".to_string(),
                default_model: "deepseek-v4-flash".to_string(),
            },
        );

        AppConfig {
            bind_addr,
            default_provider,
            log_prompts,
            upstream_timeout_secs,
            providers,
        }
    }

    pub fn get_api_key(&self, provider_key: &str) -> Option<String> {
        std::env::var(provider_key).ok()
    }

    pub fn get_opencode_auth_json_path() -> Option<PathBuf> {
        let data_dir = std::env::var("XDG_DATA_HOME")
            .map(PathBuf::from)
            .ok()
            .or_else(|| {
                dirs::home_dir().map(|h| h.join(".local").join("share"))
            });

        data_dir.map(|d| d.join("opencode").join("auth.json"))
    }

    pub fn read_opencode_api_key() -> Option<String> {
        if let Ok(content) = std::env::var("OPENCODE_AUTH_CONTENT") {
            let parsed: serde_json::Value = serde_json::from_str(&content).ok()?;
            for key in ["opencode-go", "opencode"] {
                if let Some(entry) = parsed.get(key) {
                    if let Some(api_key) = Self::extract_key_from_auth_entry(entry) {
                        return Some(api_key);
                    }
                }
            }
        }

        let path = Self::get_opencode_auth_json_path()?;
        if !path.exists() {
            return None;
        }
        let content = std::fs::read_to_string(&path).ok()?;
        let parsed: serde_json::Value = serde_json::from_str(&content).ok()?;
        for key in ["opencode-go", "opencode"] {
            if let Some(entry) = parsed.get(key) {
                if let Some(api_key) = Self::extract_key_from_auth_entry(entry) {
                    return Some(api_key);
                }
            }
        }
        None
    }

    fn extract_key_from_auth_entry(entry: &serde_json::Value) -> Option<String> {
        match entry.get("type")?.as_str()? {
            "api" => entry.get("key").and_then(|v| v.as_str()).map(|s| s.to_string()),
            "oauth" => entry.get("access").and_then(|v| v.as_str()).map(|s| s.to_string()),
            "wellknown" => entry.get("token").and_then(|v| v.as_str()).map(|s| s.to_string()),
            _ => None,
        }
    }
}
