use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use serde_json::Value;

use crate::content::ContentType;
use crate::masking::{mask_message, unmask_response};
use crate::provider::LLMProvider;
use crate::state::{PrivacyMode, ProxyState};

/// POST /v1/chat/completions - Proxy chat completions to LLM with PII masking
pub async fn proxy_chat_completions(
    State(state): State<ProxyState>,
    headers: HeaderMap,
    Json(mut body): Json<Value>,
) -> impl IntoResponse {
    let mode = state.get_mode().await;
    let privacy_guard_url = state.privacy_guard_url.clone();
    
    // Extract content type from request
    let content_type_str = headers
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("application/json");
    
    // Task B.6: Detect content type and check if maskable
    let content_type = ContentType::from_header(content_type_str);
    let is_maskable = content_type.is_maskable();
    
    // Log the request with content type information
    state.log_activity(
        "chat_completion",
        content_type_str,
        format!("Mode: {}, ContentType: {}, Maskable: {}", mode, content_type.name(), is_maskable),
    ).await;
    
    // Task B.6: Mode enforcement based on content type
    // Strict mode + non-maskable content → error
    if mode == PrivacyMode::Strict && !is_maskable {
        state.log_activity(
            "strict_mode_blocked",
            content_type_str,
            format!("Strict mode blocks non-maskable content type: {}", content_type.name()),
        ).await;
        
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": {
                    "message": format!(
                        "Strict mode does not allow non-maskable content type '{}'. Use Auto or Bypass mode for {}/*, application/pdf, etc.",
                        content_type.name(),
                        content_type.name()
                    ),
                    "type": "content_type_not_allowed",
                    "content_type": content_type.name()
                }
            })),
        ).into_response();
    }
    
    // Auto mode + non-maskable content → pass-through with warning
    if mode == PrivacyMode::Auto && !is_maskable {
        state.log_activity(
            "auto_mode_passthrough",
            content_type_str,
            format!("Auto mode bypassing non-maskable content type: {}", content_type.name()),
        ).await;
        
        // Forward request without masking
        let (provider_url, endpoint) = match detect_provider(&headers) {
            Ok(provider) => {
                (provider.base_url().to_string(), provider.chat_completions_endpoint().to_string())
            }
            Err(_) => {
                let base = std::env::var("LLM_PROVIDER_URL")
                    .unwrap_or_else(|_| "https://openrouter.ai/api".to_string());
                (base, "/v1/chat/completions".to_string())
            }
        };
        
        return match forward_request(&provider_url, &endpoint, body, &headers).await {
            Ok(response) => {
                state.log_activity(
                    "passthrough_success",
                    content_type_str,
                    format!("Non-maskable content forwarded ({})", content_type.name()),
                ).await;
                (StatusCode::OK, Json(response)).into_response()
            }
            Err(e) => {
                state.log_activity(
                    "passthrough_error",
                    content_type_str,
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
        };
    }
    
    // Task B.2: Add masking logic based on mode (for maskable content)
    // Use "proxy" as tenant_id for all requests
    let tenant_id = "proxy";
    let masking_session_id = match mode {
        PrivacyMode::Auto | PrivacyMode::Strict => {
            // Mask messages before sending to LLM
            match mask_messages(&privacy_guard_url, &mut body, tenant_id).await {
                Ok(session_id) => {
                    state.log_activity(
                        "masking_success",
                        content_type_str,
                        format!("Messages masked, session_id: {}", session_id),
                    ).await;
                    Some(session_id)
                }
                Err(e) => {
                    state.log_activity(
                        "masking_error",
                        content_type_str,
                        format!("Masking failed: {}", e),
                    ).await;
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(serde_json::json!({
                            "error": {
                                "message": format!("Failed to mask PII: {}", e),
                                "type": "masking_error"
                            }
                        })),
                    ).into_response();
                }
            }
        }
        PrivacyMode::Bypass => {
            // Bypass mode: No masking, log for audit
            state.log_activity(
                "bypass_mode",
                content_type_str,
                "PII masking bypassed by user",
            ).await;
            None
        }
    };
    
    // Detect provider and build URL
    let (provider_url, endpoint) = match detect_provider(&headers) {
        Ok(provider) => {
            let url = provider.chat_completions_url();
            state.log_activity(
                "provider_detected",
                content_type_str,
                format!("Provider: {}, URL: {}", provider.name(), url),
            ).await;
            (provider.base_url().to_string(), provider.chat_completions_endpoint().to_string())
        }
        Err(e) => {
            state.log_activity(
                "provider_detection_error",
                content_type_str,
                format!("Failed to detect provider: {}, using default", e),
            ).await;
            // Fallback to env var or default
            let base = std::env::var("LLM_PROVIDER_URL")
                .unwrap_or_else(|_| "https://openrouter.ai/api".to_string());
            (base, "/v1/chat/completions".to_string())
        }
    };
    
    // Forward the request to the LLM provider
    match forward_request(&provider_url, &endpoint, body, &headers).await {
        Ok(mut response) => {
            // Unmask response if we have a session_id
            if let Some(session_id) = masking_session_id {
                if let Some(response_content) = extract_response_content(&mut response) {
                    let client = reqwest::Client::new();
                    match unmask_response(&privacy_guard_url, &response_content, tenant_id, &session_id, &client).await {
                        Ok(unmasked) => {
                            update_response_content(&mut response, unmasked);
                            state.log_activity(
                                "unmasking_success",
                                content_type_str,
                                "Response unmasked successfully",
                            ).await;
                        }
                        Err(e) => {
                            state.log_activity(
                                "unmasking_error",
                                content_type_str,
                                format!("Unmasking failed (returning masked response): {}", e),
                            ).await;
                            // Return masked response rather than error
                        }
                    }
                }
            }
            
            state.log_activity(
                "chat_completion_success",
                content_type_str,
                "Request completed successfully",
            ).await;
            
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => {
            state.log_activity(
                "chat_completion_error",
                content_type_str,
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
    
    // Detect provider and build URL
    let (provider_url, endpoint) = match detect_provider(&headers) {
        Ok(provider) => {
            (provider.base_url().to_string(), provider.completions_endpoint().to_string())
        }
        Err(_) => {
            let base = std::env::var("LLM_PROVIDER_URL")
                .unwrap_or_else(|_| "https://openrouter.ai/api".to_string());
            (base, "/v1/completions".to_string())
        }
    };
    
    match forward_request(&provider_url, &endpoint, body, &headers).await {
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

/// Determine the LLM provider from API key in headers
fn detect_provider(headers: &HeaderMap) -> Result<LLMProvider, String> {
    // Extract Authorization header
    let auth_header = headers
        .get("authorization")
        .or_else(|| headers.get("Authorization"))
        .ok_or_else(|| "Missing Authorization header".to_string())?;
    
    let auth_str = auth_header
        .to_str()
        .map_err(|_| "Invalid Authorization header format".to_string())?;
    
    // Extract API key from "Bearer sk-..." format
    let api_key = if auth_str.starts_with("Bearer ") {
        &auth_str[7..] // Skip "Bearer "
    } else {
        auth_str
    };
    
    Ok(LLMProvider::from_api_key(api_key))
}

/// Forward a request to the LLM provider
/// Task B.3: Now uses API key from headers (not environment)
async fn forward_request(
    base_url: &str,
    path: &str,
    body: Value,
    headers: &HeaderMap,
) -> Result<Value, String> {
    let client = reqwest::Client::new();
    let url = format!("{}{}", base_url, path);
    
    // Extract Authorization header from incoming request
    let auth_header = headers
        .get("authorization")
        .or_else(|| headers.get("Authorization"))
        .ok_or_else(|| "Missing Authorization header".to_string())?;
    
    let auth_str = auth_header
        .to_str()
        .map_err(|_| "Invalid Authorization header".to_string())?;
    
    // Forward request with original Authorization header
    let response = client
        .post(&url)
        .header("Authorization", auth_str)
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

/// Mask all messages in a chat completion request
/// Returns session_id from Privacy Guard (used for reidentification)
async fn mask_messages(privacy_guard_url: &str, body: &mut Value, tenant_id: &str) -> Result<String, String> {
    let client = reqwest::Client::new();
    let mut last_session_id = String::new();
    
    // Extract messages array from request body
    if let Some(messages) = body.get_mut("messages").and_then(|m| m.as_array_mut()) {
        for message in messages.iter_mut() {
            if let Some(content) = message.get("content").and_then(|c| c.as_str()) {
                // Mask this message
                let (masked, session_id) = mask_message(privacy_guard_url, content, tenant_id, &client).await?;
                
                // Update message content with masked version
                message["content"] = Value::String(masked);
                
                // Store session_id (all messages in same request use same session)
                last_session_id = session_id;
            }
        }
    }
    
    Ok(last_session_id)
}

/// Extract response content from LLM response
fn extract_response_content(response: &Value) -> Option<String> {
    response
        .get("choices")?
        .get(0)?
        .get("message")?
        .get("content")?
        .as_str()
        .map(|s| s.to_string())
}

/// Update response content with unmasked text
fn update_response_content(response: &mut Value, unmasked: String) {
    if let Some(choices) = response.get_mut("choices").and_then(|c| c.as_array_mut()) {
        if let Some(first_choice) = choices.get_mut(0) {
            if let Some(message) = first_choice.get_mut("message") {
                message["content"] = Value::String(unmasked);
            }
        }
    }
}
