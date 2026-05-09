use crate::config::AppConfig;
use crate::error::AppError;

pub fn route_provider(model: &str, config: &AppConfig) -> Result<String, AppError> {
    if model.starts_with("deepseek") || model.starts_with("deepseek-") {
        return Ok("deepseek".to_string());
    }
    if model.starts_with("MiniMax") || model.starts_with("minimax") || model.starts_with("codex-MiniMax") {
        return Ok("minimax".to_string());
    }
    if config.providers.contains_key(&config.default_provider) {
        return Ok(config.default_provider.clone());
    }
    Err(AppError::bad_request(format!(
        "no provider configured for model '{model}' and DEFAULT_PROVIDER is invalid"
    )))
}

pub fn resolve_provider(
    model: &str,
    config: &AppConfig,
) -> Result<(String, crate::config::ProviderConfig), AppError> {
    let provider_name = route_provider(model, config)?;
    let provider_cfg = config.providers.get(&provider_name).ok_or_else(|| {
        AppError::bad_request(format!("provider '{provider_name}' not found in config"))
    })?;
    let api_key = config.get_api_key(&provider_cfg.api_key_env).ok_or_else(|| {
        AppError::bad_request(format!(
            "API key missing: set {} environment variable",
            provider_cfg.api_key_env
        ))
    })?;
    Ok((api_key, provider_cfg.clone()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> AppConfig {
        let mut c = AppConfig::from_env();
        c.default_provider = "deepseek".to_string();
        c
    }

    #[test]
    fn test_route_deepseek_chat() {
        let c = test_config();
        assert_eq!(route_provider("deepseek-chat", &c).unwrap(), "deepseek");
    }

    #[test]
    fn test_route_deepseek_reasoner() {
        let c = test_config();
        assert_eq!(route_provider("deepseek-reasoner", &c).unwrap(), "deepseek");
    }

    #[test]
    fn test_route_minimax() {
        let c = test_config();
        assert_eq!(route_provider("codex-MiniMax-M2.7", &c).unwrap(), "minimax");
        assert_eq!(route_provider("MiniMax-M2.7", &c).unwrap(), "minimax");
        assert_eq!(route_provider("minimax-anything", &c).unwrap(), "minimax");
    }

    #[test]
    fn test_route_default() {
        let c = test_config();
        assert_eq!(route_provider("unknown-model", &c).unwrap(), "deepseek");
    }
}
