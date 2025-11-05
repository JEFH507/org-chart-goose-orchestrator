// RBAC/ABAC Policy Engine
// Phase 5 Workstream C: Task C1
//
// Evaluates role-based policies for tool usage and data access.
// Implements caching via Redis for performance (5 min TTL).
// Defaults to deny for security-first approach.

use anyhow::{Context, Result};
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::collections::HashMap;
use thiserror::Error;

/// Policy evaluation context (for ABAC conditions)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyContext {
    /// Database name (for sql-mcp__query conditions)
    pub database: Option<String>,
    /// File path (for developer__* conditions)
    pub file_path: Option<String>,
    /// Additional context fields
    pub extra: HashMap<String, String>,
}

impl PolicyContext {
    /// Create empty context
    pub fn empty() -> Self {
        Self {
            database: None,
            file_path: None,
            extra: HashMap::new(),
        }
    }

    /// Create context with database name
    pub fn with_database(database: &str) -> Self {
        Self {
            database: Some(database.to_string()),
            file_path: None,
            extra: HashMap::new(),
        }
    }
}

/// Policy evaluation errors
#[derive(Error, Debug)]
pub enum PolicyError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Redis error: {0}")]
    Redis(#[from] redis::RedisError),

    #[error("Policy denied: {0}")]
    Denied(String),

    #[error("Internal error: {0}")]
    Internal(#[from] anyhow::Error),
}

/// Policy record from database
#[derive(Debug, Clone, sqlx::FromRow)]
struct Policy {
    role: String,
    tool_pattern: String,
    allow: bool,
    conditions: Option<sqlx::types::Json<HashMap<String, String>>>,
    reason: Option<String>,
}

impl Policy {
    /// Check if policy matches the tool name (supports glob patterns)
    fn matches_tool(&self, tool_name: &str) -> bool {
        // Simple glob matching: "github__*" matches "github__list_issues"
        if self.tool_pattern.ends_with('*') {
            let prefix = self.tool_pattern.trim_end_matches('*');
            tool_name.starts_with(prefix)
        } else {
            tool_name == self.tool_pattern
        }
    }

    /// Check if ABAC conditions are met
    fn conditions_met(&self, context: &PolicyContext) -> bool {
        let Some(conditions) = &self.conditions else {
            // No conditions = always match
            return true;
        };

        // Check database condition (for sql-mcp__query)
        if let Some(db_pattern) = conditions.get("database") {
            let Some(db_name) = &context.database else {
                return false; // Required condition missing
            };

            // Support glob patterns: "analytics_*"
            if db_pattern.ends_with('*') {
                let prefix = db_pattern.trim_end_matches('*');
                if !db_name.starts_with(prefix) {
                    return false;
                }
            } else if db_name != db_pattern {
                return false;
            }
        }

        // All conditions met
        true
    }
}

/// Policy engine for RBAC/ABAC evaluation
pub struct PolicyEngine {
    postgres_pool: PgPool,
    redis_client: Option<redis::aio::ConnectionManager>,
}

impl PolicyEngine {
    /// Create new policy engine
    pub fn new(
        postgres_pool: PgPool,
        redis_client: Option<redis::aio::ConnectionManager>,
    ) -> Self {
        Self {
            postgres_pool,
            redis_client,
        }
    }

    /// Evaluate if role can use tool
    ///
    /// Returns true if allowed, false if denied.
    /// Uses Redis cache with 300s TTL for performance.
    /// Defaults to deny if no policy found (security-first).
    pub async fn can_use_tool(
        &self,
        role: &str,
        tool_name: &str,
        context: &PolicyContext,
    ) -> Result<bool, PolicyError> {
        let cache_key = format!("policy:{}:{}", role, tool_name);

        // 1. Check cache (if Redis available)
        if let Some(redis) = &self.redis_client {
            let mut conn = redis.clone();
            if let Ok(cached) = conn.get::<_, Option<String>>(&cache_key).await {
                if let Some(value) = cached {
                    // Cache hit
                    return Ok(value == "allow");
                }
            }
        }

        // 2. Load policies from database
        let policies = sqlx::query_as::<_, Policy>(
            "SELECT role, tool_pattern, allow, conditions, reason 
             FROM policies 
             WHERE role = $1 
             ORDER BY tool_pattern DESC", // Most specific first
        )
        .bind(role)
        .fetch_all(&self.postgres_pool)
        .await
        .context("Failed to load policies from database")?;

        // 3. Evaluate policies
        let mut result = false; // Deny by default
        let mut matched = false;

        for policy in policies {
            if policy.matches_tool(tool_name) && policy.conditions_met(context) {
                result = policy.allow;
                matched = true;
                break; // First match wins
            }
        }

        // 4. Cache result (if Redis available)
        if let Some(redis) = &self.redis_client {
            let mut conn = redis.clone();
            let cache_value = if result { "allow" } else { "deny" };
            let _ = conn
                .set_ex::<_, _, ()>(&cache_key, cache_value, 300) // 5 min TTL
                .await;
        }

        // 5. Return result (deny by default if no match)
        if !matched {
            // No policy found - deny by default
            return Err(PolicyError::Denied(format!(
                "No policy found for role '{}' and tool '{}' (default deny)",
                role, tool_name
            )));
        }

        Ok(result)
    }

    /// Evaluate if role can access data
    ///
    /// Similar to can_use_tool but for data access policies.
    /// Currently a placeholder for future data access controls.
    pub async fn can_access_data(
        &self,
        role: &str,
        data_type: &str,
        context: &PolicyContext,
    ) -> Result<bool, PolicyError> {
        // For now, delegate to can_use_tool with data_type as tool_name
        // Future: separate data_policies table
        self.can_use_tool(role, &format!("data_access__{}", data_type), context)
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_policy_matches_tool() {
        let policy = Policy {
            role: "finance".to_string(),
            tool_pattern: "github__*".to_string(),
            allow: true,
            conditions: None,
            reason: None,
        };

        assert!(policy.matches_tool("github__list_issues"));
        assert!(policy.matches_tool("github__create_issue"));
        assert!(!policy.matches_tool("developer__shell"));
    }

    #[test]
    fn test_policy_exact_match() {
        let policy = Policy {
            role: "finance".to_string(),
            tool_pattern: "developer__shell".to_string(),
            allow: false,
            conditions: None,
            reason: Some("No code execution".to_string()),
        };

        assert!(policy.matches_tool("developer__shell"));
        assert!(!policy.matches_tool("developer__git"));
    }

    #[test]
    fn test_policy_conditions_database() {
        let mut conditions = HashMap::new();
        conditions.insert("database".to_string(), "analytics_*".to_string());

        let policy = Policy {
            role: "analyst".to_string(),
            tool_pattern: "sql-mcp__query".to_string(),
            allow: true,
            conditions: Some(sqlx::types::Json(conditions)),
            reason: None,
        };

        // Match
        let ctx = PolicyContext::with_database("analytics_prod");
        assert!(policy.conditions_met(&ctx));

        // No match
        let ctx = PolicyContext::with_database("finance_db");
        assert!(!policy.conditions_met(&ctx));

        // Missing context
        let ctx = PolicyContext::empty();
        assert!(!policy.conditions_met(&ctx));
    }

    #[test]
    fn test_policy_no_conditions() {
        let policy = Policy {
            role: "manager".to_string(),
            tool_pattern: "agent_mesh__*".to_string(),
            allow: true,
            conditions: None,
            reason: None,
        };

        // Always match when no conditions
        let ctx = PolicyContext::empty();
        assert!(policy.conditions_met(&ctx));
    }
}
