// provider.rs - LLM provider detection and routing

use serde::{Deserialize, Serialize};

/// Supported LLM providers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LLMProvider {
    OpenRouter,
    Anthropic,
    OpenAI,
}

impl LLMProvider {
    /// Detect provider from API key format
    ///
    /// - sk-or-* → OpenRouter
    /// - sk-ant-* → Anthropic  
    /// - sk-* → OpenAI
    pub fn from_api_key(api_key: &str) -> Self {
        if api_key.starts_with("sk-or-") {
            LLMProvider::OpenRouter
        } else if api_key.starts_with("sk-ant-") {
            LLMProvider::Anthropic
        } else if api_key.starts_with("sk-") {
            LLMProvider::OpenAI
        } else {
            // Default to OpenRouter for unknown formats
            LLMProvider::OpenRouter
        }
    }

    /// Get the base URL for this provider
    pub fn base_url(&self) -> &'static str {
        match self {
            LLMProvider::OpenRouter => "https://openrouter.ai/api",
            LLMProvider::Anthropic => "https://api.anthropic.com",
            LLMProvider::OpenAI => "https://api.openai.com",
        }
    }

    /// Get the chat completions endpoint for this provider
    pub fn chat_completions_endpoint(&self) -> &'static str {
        match self {
            LLMProvider::OpenRouter => "/v1/chat/completions",
            LLMProvider::Anthropic => "/v1/messages", // Anthropic uses different endpoint
            LLMProvider::OpenAI => "/v1/chat/completions",
        }
    }

    /// Get the completions endpoint for this provider (legacy)
    pub fn completions_endpoint(&self) -> &'static str {
        match self {
            LLMProvider::OpenRouter => "/v1/completions",
            LLMProvider::Anthropic => "/v1/completions", // May not be supported
            LLMProvider::OpenAI => "/v1/completions",
        }
    }

    /// Check if this provider uses OpenAI-compatible schema
    pub fn is_openai_compatible(&self) -> bool {
        matches!(self, LLMProvider::OpenRouter | LLMProvider::OpenAI)
    }

    /// Get the full URL for chat completions
    pub fn chat_completions_url(&self) -> String {
        format!("{}{}", self.base_url(), self.chat_completions_endpoint())
    }

    /// Get the full URL for completions (legacy)
    pub fn completions_url(&self) -> String {
        format!("{}{}", self.base_url(), self.completions_endpoint())
    }

    /// Get provider name as string
    pub fn name(&self) -> &'static str {
        match self {
            LLMProvider::OpenRouter => "OpenRouter",
            LLMProvider::Anthropic => "Anthropic",
            LLMProvider::OpenAI => "OpenAI",
        }
    }
}

impl std::fmt::Display for LLMProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_openrouter() {
        let api_key = "sk-or-v1-1234567890abcdef";
        assert_eq!(LLMProvider::from_api_key(api_key), LLMProvider::OpenRouter);
    }

    #[test]
    fn test_detect_anthropic() {
        let api_key = "sk-ant-api03-1234567890abcdef";
        assert_eq!(LLMProvider::from_api_key(api_key), LLMProvider::Anthropic);
    }

    #[test]
    fn test_detect_openai() {
        let api_key = "sk-proj-1234567890abcdef";
        assert_eq!(LLMProvider::from_api_key(api_key), LLMProvider::OpenAI);
        
        let api_key2 = "sk-1234567890abcdef";
        assert_eq!(LLMProvider::from_api_key(api_key2), LLMProvider::OpenAI);
    }

    #[test]
    fn test_unknown_defaults_to_openrouter() {
        let api_key = "unknown-format-key";
        assert_eq!(LLMProvider::from_api_key(api_key), LLMProvider::OpenRouter);
    }

    #[test]
    fn test_openrouter_urls() {
        let provider = LLMProvider::OpenRouter;
        assert_eq!(provider.base_url(), "https://openrouter.ai/api");
        assert_eq!(provider.chat_completions_endpoint(), "/v1/chat/completions");
        assert_eq!(provider.chat_completions_url(), "https://openrouter.ai/api/v1/chat/completions");
    }

    #[test]
    fn test_anthropic_urls() {
        let provider = LLMProvider::Anthropic;
        assert_eq!(provider.base_url(), "https://api.anthropic.com");
        assert_eq!(provider.chat_completions_endpoint(), "/v1/messages");
        assert_eq!(provider.chat_completions_url(), "https://api.anthropic.com/v1/messages");
    }

    #[test]
    fn test_openai_urls() {
        let provider = LLMProvider::OpenAI;
        assert_eq!(provider.base_url(), "https://api.openai.com");
        assert_eq!(provider.chat_completions_endpoint(), "/v1/chat/completions");
        assert_eq!(provider.chat_completions_url(), "https://api.openai.com/v1/chat/completions");
    }

    #[test]
    fn test_openai_compatible() {
        assert!(LLMProvider::OpenRouter.is_openai_compatible());
        assert!(LLMProvider::OpenAI.is_openai_compatible());
        assert!(!LLMProvider::Anthropic.is_openai_compatible());
    }

    #[test]
    fn test_provider_names() {
        assert_eq!(LLMProvider::OpenRouter.name(), "OpenRouter");
        assert_eq!(LLMProvider::Anthropic.name(), "Anthropic");
        assert_eq!(LLMProvider::OpenAI.name(), "OpenAI");
    }

    #[test]
    fn test_provider_display() {
        assert_eq!(format!("{}", LLMProvider::OpenRouter), "OpenRouter");
        assert_eq!(format!("{}", LLMProvider::Anthropic), "Anthropic");
        assert_eq!(format!("{}", LLMProvider::OpenAI), "OpenAI");
    }
}
