use std::collections::HashMap;

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
}
