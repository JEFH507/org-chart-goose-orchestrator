// Policy Enforcement Middleware
// Phase 5 Workstream C: Task C4
//
// Enforces RBAC/ABAC policies before allowing requests to routes.
// Extracts role from JWT, tool name from request, evaluates policy.
// Returns 403 Forbidden if policy denies access.

use axum::{
    body::Body,
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use std::sync::Arc;
use tracing::{debug, warn};

use crate::policy::{PolicyContext, PolicyEngine, PolicyError};
use crate::AppState;

/// Policy enforcement middleware
///
/// Extracts role from JWT claims and tool name from request.
/// Evaluates policy using PolicyEngine.
/// Returns 403 Forbidden if access denied.
///
/// Note: This middleware should be applied AFTER JWT middleware
/// so that JWT claims are already validated and available.
pub async fn enforce_policy(
    State(state): State<AppState>,
    mut req: Request,
    next: Next,
) -> Result<Response, PolicyDeniedResponse> {
    // 1. Extract role from JWT claims (set by JWT middleware)
    let role = match req.extensions().get::<String>() {
        Some(role_claim) => role_claim.clone(),
        None => {
            // No role claim found - might be unauthenticated route
            // Skip policy enforcement for unauthenticated routes
            debug!("No role claim found, skipping policy enforcement");
            return Ok(next.run(req).await);
        }
    };

    // 2. Extract tool name from request
    // For now, we'll extract from the request path or body
    // This is a placeholder - actual extraction depends on API design
    let tool_name = extract_tool_name(&req).await;
    
    let Some(tool_name) = tool_name else {
        // No tool name in request - skip policy enforcement
        debug!(role = %role, "No tool name found, skipping policy enforcement");
        return Ok(next.run(req).await);
    };

    // 3. Build policy context (for ABAC conditions)
    let context = extract_policy_context(&req).await;

    // 4. Check policy using PolicyEngine
    let policy_engine = create_policy_engine(&state)?;
    
    match policy_engine.can_use_tool(&role, &tool_name, &context).await {
        Ok(true) => {
            // Policy allows - proceed to route
            debug!(role = %role, tool = %tool_name, "Policy allows access");
            Ok(next.run(req).await)
        }
        Ok(false) => {
            // Policy explicitly denies
            warn!(role = %role, tool = %tool_name, "Policy denies access");
            Err(PolicyDeniedResponse {
                role: role.clone(),
                tool: tool_name.clone(),
                reason: format!("Policy denies {} access to {}", role, tool_name),
            })
        }
        Err(PolicyError::Denied(reason)) => {
            // No policy found - default deny
            warn!(role = %role, tool = %tool_name, "No policy found, default deny");
            Err(PolicyDeniedResponse {
                role: role.clone(),
                tool: tool_name.clone(),
                reason,
            })
        }
        Err(e) => {
            // Policy evaluation error - fail closed (deny access)
            warn!(role = %role, tool = %tool_name, error = %e, "Policy evaluation error, denying access");
            Err(PolicyDeniedResponse {
                role: role.clone(),
                tool: tool_name.clone(),
                reason: format!("Policy evaluation error: {}", e),
            })
        }
    }
}

/// Extract tool name from request
///
/// Checks request path and body for tool references.
/// Returns None if no tool name found.
async fn extract_tool_name(req: &Request) -> Option<String> {
    // Strategy 1: Check path for tool routes
    // Example: POST /tools/developer__shell → "developer__shell"
    let path = req.uri().path();
    if let Some(tool_segment) = path.strip_prefix("/tools/") {
        return Some(tool_segment.to_string());
    }

    // Strategy 2: Check for task routing requests
    // Example: POST /tasks/route with body containing tool name
    if path == "/tasks/route" {
        // TODO: Parse request body for tool name
        // For now, return None (body parsing requires more complex logic)
        // This will be implemented when task routing is fully designed
        return None;
    }

    // Strategy 3: Check custom header
    // Example: X-Tool-Name: developer__shell
    if let Some(tool_header) = req.headers().get("X-Tool-Name") {
        if let Ok(tool_str) = tool_header.to_str() {
            return Some(tool_str.to_string());
        }
    }

    None
}

/// Extract policy context from request
///
/// Builds PolicyContext with ABAC attributes extracted from request.
async fn extract_policy_context(req: &Request) -> PolicyContext {
    let mut context = PolicyContext::empty();

    // Extract database name from query params or headers
    if let Some(db_header) = req.headers().get("X-Database-Name") {
        if let Ok(db_str) = db_header.to_str() {
            context.database = Some(db_str.to_string());
        }
    }

    // Extract file path from query params or headers
    if let Some(path_header) = req.headers().get("X-File-Path") {
        if let Ok(path_str) = path_header.to_str() {
            context.file_path = Some(path_str.to_string());
        }
    }

    context
}

/// Create PolicyEngine from AppState
fn create_policy_engine(state: &AppState) -> Result<PolicyEngine, PolicyDeniedResponse> {
    let db_pool = state.db_pool.clone().ok_or_else(|| PolicyDeniedResponse {
        role: "unknown".to_string(),
        tool: "unknown".to_string(),
        reason: "Database not configured".to_string(),
    })?;

    Ok(PolicyEngine::new(db_pool, state.redis_client.clone()))
}

/// Policy denied response
#[derive(Debug)]
struct PolicyDeniedResponse {
    role: String,
    tool: String,
    reason: String,
}

impl IntoResponse for PolicyDeniedResponse {
    fn into_response(self) -> Response {
        let body = json!({
            "error": "Policy Denied",
            "role": self.role,
            "tool": self.tool,
            "reason": self.reason,
            "status": 403,
        });

        (StatusCode::FORBIDDEN, Json(body)).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::Request;

    #[tokio::test]
    async fn test_extract_tool_name_from_path() {
        let req = Request::builder()
            .uri("/tools/developer__shell")
            .body(Body::empty())
            .unwrap();

        let tool = extract_tool_name(&req).await;
        assert_eq!(tool, Some("developer__shell".to_string()));
    }

    #[tokio::test]
    async fn test_extract_tool_name_from_header() {
        let req = Request::builder()
            .uri("/some/path")
            .header("X-Tool-Name", "github__list_issues")
            .body(Body::empty())
            .unwrap();

        let tool = extract_tool_name(&req).await;
        assert_eq!(tool, Some("github__list_issues".to_string()));
    }

    #[tokio::test]
    async fn test_extract_tool_name_not_found() {
        let req = Request::builder()
            .uri("/status")
            .body(Body::empty())
            .unwrap();

        let tool = extract_tool_name(&req).await;
        assert_eq!(tool, None);
    }

    #[tokio::test]
    async fn test_extract_policy_context() {
        let req = Request::builder()
            .uri("/some/path")
            .header("X-Database-Name", "analytics_prod")
            .header("X-File-Path", "/data/reports/q4.csv")
            .body(Body::empty())
            .unwrap();

        let context = extract_policy_context(&req).await;
        assert_eq!(context.database, Some("analytics_prod".to_string()));
        assert_eq!(context.file_path, Some("/data/reports/q4.csv".to_string()));
    }
}
