// Privacy Guard MCP Extension
// Provides local PII protection for Goose agents via stdio MCP protocol
//
// Architecture:
//   Goose Client → Privacy Guard MCP → LLM Provider
//                     ↓ (audit)
//                  Controller API

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::io::{self, BufRead, Write};
use tracing::{error, info, warn};

mod config;
mod interceptor;
mod ollama;
mod redaction;
mod tokenizer;

use config::Config;
use interceptor::{RequestInterceptor, ResponseInterceptor};

/// MCP Request from Goose client
#[derive(Debug, Deserialize)]
struct McpRequest {
    jsonrpc: String,
    id: Option<serde_json::Value>,
    method: String,
    params: Option<serde_json::Value>,
}

/// MCP Response to Goose client
#[derive(Debug, Serialize)]
struct McpResponse {
    jsonrpc: String,
    id: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<McpError>,
}

#[derive(Debug, Serialize)]
struct McpError {
    code: i32,
    message: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .with_writer(std::io::stderr) // Log to stderr (stdout reserved for MCP protocol)
        .init();

    info!("Privacy Guard MCP Extension starting...");

    // Load configuration
    let config = Config::from_env().context("Failed to load configuration")?;
    info!("Configuration loaded: mode={:?}, strictness={:?}", config.mode, config.strictness);

    // Initialize interceptors
    let request_interceptor = RequestInterceptor::new(config.clone())?;
    let response_interceptor = ResponseInterceptor::new(config.clone())?;

    info!("Privacy Guard MCP ready (stdio mode)");

    // MCP stdio protocol loop
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let reader = stdin.lock();

    for line in reader.lines() {
        let line = match line {
            Ok(l) => l,
            Err(e) => {
                error!("Failed to read stdin: {}", e);
                continue;
            }
        };

        // Parse MCP request
        let request: McpRequest = match serde_json::from_str(&line) {
            Ok(req) => req,
            Err(e) => {
                error!("Failed to parse MCP request: {}", e);
                let error_response = McpResponse {
                    jsonrpc: "2.0".to_string(),
                    id: None,
                    result: None,
                    error: Some(McpError {
                        code: -32700,
                        message: format!("Parse error: {}", e),
                    }),
                };
                if let Ok(json) = serde_json::to_string(&error_response) {
                    writeln!(stdout, "{}", json).ok();
                    stdout.flush().ok();
                }
                continue;
            }
        };

        // Handle MCP methods
        let response = match request.method.as_str() {
            "initialize" => handle_initialize(request).await,
            "tools/list" => handle_tools_list(request).await,
            "tools/call" => {
                handle_tools_call(
                    request,
                    &request_interceptor,
                    &response_interceptor,
                )
                .await
            }
            "shutdown" => {
                info!("Shutdown requested");
                break;
            }
            _ => {
                warn!("Unknown method: {}", request.method);
                McpResponse {
                    jsonrpc: "2.0".to_string(),
                    id: request.id,
                    result: None,
                    error: Some(McpError {
                        code: -32601,
                        message: format!("Method not found: {}", request.method),
                    }),
                }
            }
        };

        // Send response
        let json = serde_json::to_string(&response).context("Failed to serialize response")?;
        writeln!(stdout, "{}", json)?;
        stdout.flush()?;
    }

    info!("Privacy Guard MCP shutting down");
    Ok(())
}

/// Handle MCP initialize request
async fn handle_initialize(request: McpRequest) -> McpResponse {
    info!("Initialize request received");
    McpResponse {
        jsonrpc: "2.0".to_string(),
        id: request.id,
        result: Some(serde_json::json!({
            "protocolVersion": "2024-11-05",
            "serverInfo": {
                "name": "privacy-guard-mcp",
                "version": env!("CARGO_PKG_VERSION")
            },
            "capabilities": {
                "tools": {}
            }
        })),
        error: None,
    }
}

/// Handle MCP tools/list request
async fn handle_tools_list(request: McpRequest) -> McpResponse {
    info!("Tools list request received");
    McpResponse {
        jsonrpc: "2.0".to_string(),
        id: request.id,
        result: Some(serde_json::json!({
            "tools": [
                {
                    "name": "apply_privacy_guard",
                    "description": "Apply privacy protection to prompts before sending to LLM",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "prompt": {
                                "type": "string",
                                "description": "The prompt to protect"
                            },
                            "session_id": {
                                "type": "string",
                                "description": "Session identifier for token tracking"
                            }
                        },
                        "required": ["prompt", "session_id"]
                    }
                }
            ]
        })),
        error: None,
    }
}

/// Handle MCP tools/call request (intercept and protect)
async fn handle_tools_call(
    request: McpRequest,
    request_interceptor: &RequestInterceptor,
    response_interceptor: &ResponseInterceptor,
) -> McpResponse {
    info!("Tool call request received");

    // Extract params
    let params = match request.params {
        Some(p) => p,
        None => {
            return McpResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id,
                result: None,
                error: Some(McpError {
                    code: -32602,
                    message: "Missing params".to_string(),
                }),
            };
        }
    };

    // Apply privacy protection
    match request_interceptor.intercept(params).await {
        Ok(protected_params) => {
            info!("Privacy protection applied successfully");
            McpResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id,
                result: Some(protected_params),
                error: None,
            }
        }
        Err(e) => {
            error!("Failed to apply privacy protection: {}", e);
            McpResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id,
                result: None,
                error: Some(McpError {
                    code: -32603,
                    message: format!("Internal error: {}", e),
                }),
            }
        }
    }
}
