// Library exports for testing
pub mod auth;
pub mod guard_client;
pub mod api;
pub mod routes;

use std::sync::Arc;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use crate::guard_client::GuardClient;
use crate::auth::JwtConfig;

#[derive(Clone)]
pub struct AppState {
    pub guard_client: Arc<GuardClient>,
    pub jwt_config: Option<JwtConfig>,
}

impl AppState {
    pub fn new(guard_client: Arc<GuardClient>, jwt_config: Option<JwtConfig>) -> Self {
        Self {
            guard_client,
            jwt_config,
        }
    }
}

// Re-export types needed by OpenAPI (these are duplicated from main.rs for library access)
#[derive(Serialize, ToSchema)]
pub struct StatusResponse<'a> {
    pub status: &'a str,
    pub version: &'a str,
}

#[derive(Deserialize, Serialize, Debug, ToSchema)]
pub struct AuditEvent {
    pub source: String,
    pub category: String,
    pub action: String,
    pub subject: Option<String>,
    #[serde(rename = "traceId")]
    pub trace_id: Option<String>,
    pub timestamp: Option<String>,
    pub metadata: Option<serde_json::Value>,
    #[serde(default)]
    pub content: Option<String>,
}
