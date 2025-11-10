use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use serde_json::Value;

use crate::state::ProxyState;

/// POST /v1/chat/completions - Proxy chat completions to LLM
/// For Task B.1, this is a simple pass-through. Masking will be added in Task B.2.
pub async fn proxy_chat_completions(
    State(state): State<ProxyState>,
    headers: HeaderMap,
    Json(body): Json<Value>,
) -> impl IntoResponse {
    let mode = state.get_mode().await;
    
    // Extract content type from request
    let content_type = headers
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("application/json");
    
    // Log the request
    state.log_activity(
        "chat_completion",
        content_type,
        format!("Mode: {}, Provider: determining...", mode),
    ).await;
    
    // For Task B.1, we do a simple pass-through
    // Task B.2 will add the masking logic here
    
    // Determine LLM provider from API key or header
    let provider_url = determine_provider_url(&headers).await;
    
    // Forward the request to the LLM provider
    match forward_request(&provider_url, "/v1/chat/completions", body).await {
        Ok(response) => {
            state.log_activity(
                "chat_completion_success",
                content_type,
                format!("Forwarded to provider (pass-through mode for B.1)"),
            ).await;
            
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => {
            state.log_activity(
                "chat_completion_error",
                content_type,
                format!("Error: {}", e),
            ).await;
            
            (
                StatusCode::BAD_GATEWAY,
                Json(serde_json::json!({
                    "error": {
                        "message": format!("Failed to forward request: {}", e),
                        "type": "proxy_error"
                    }
                })),
            ).into_response()
        }
    }
}

/// POST /v1/completions - Proxy completions to LLM
pub async fn proxy_completions(
    State(state): State<ProxyState>,
    headers: HeaderMap,
    Json(body): Json<Value>,
) -> impl IntoResponse {
    let mode = state.get_mode().await;
    
    let content_type = headers
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("application/json");
    
    state.log_activity(
        "completion",
        content_type,
        format!("Mode: {}, Provider: determining...", mode),
    ).await;
    
    let provider_url = determine_provider_url(&headers).await;
    
    match forward_request(&provider_url, "/v1/completions", body).await {
        Ok(response) => {
            state.log_activity(
                "completion_success",
                content_type,
                "Forwarded to provider (pass-through mode for B.1)",
            ).await;
            
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => {
            state.log_activity(
                "completion_error",
                content_type,
                format!("Error: {}", e),
            ).await;
            
            (
                StatusCode::BAD_GATEWAY,
                Json(serde_json::json!({
                    "error": {
                        "message": format!("Failed to forward request: {}", e),
                        "type": "proxy_error"
                    }
                })),
            ).into_response()
        }
    }
}

/// Determine the LLM provider URL from headers
/// For Task B.1, we default to OpenRouter. Task B.3 will add provider detection.
async fn determine_provider_url(_headers: &HeaderMap) -> String {
    // For now, default to OpenRouter
    // Task B.3 will implement proper provider detection
    std::env::var("LLM_PROVIDER_URL")
        .unwrap_or_else(|_| "https://openrouter.ai".to_string())
}

/// Forward a request to the LLM provider
async fn forward_request(
    base_url: &str,
    path: &str,
    body: Value,
) -> Result<Value, String> {
    let client = reqwest::Client::new();
    let url = format!("{}{}", base_url, path);
    
    // Get API key from environment
    let api_key = std::env::var("LLM_API_KEY")
        .map_err(|_| "LLM_API_KEY not set in environment".to_string())?;
    
    let response = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;
    
    if !response.status().is_success() {
        let status = response.status();
        let error_body = response.text().await.unwrap_or_default();
        return Err(format!("Provider returned {}: {}", status, error_body));
    }
    
    response
        .json::<Value>()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))
}
