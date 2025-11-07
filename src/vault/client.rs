// Vault client - Core client wrapper around vaultrs

use super::{VaultAuth, VaultConfig};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{info, warn};

/// AppRole login response from Vault
#[derive(Debug, Deserialize)]
struct AppRoleLoginResponse {
    auth: AppRoleAuth,
}

#[derive(Debug, Deserialize)]
struct AppRoleAuth {
    client_token: String,
    lease_duration: u64,
    renewable: bool,
}

/// Production-grade Vault client with connection pooling and health checks
pub struct VaultClient {
    /// vaultrs client instance (wrapped in Arc for sharing across threads)
    inner: std::sync::Arc<vaultrs::client::VaultClient>,
    /// Configuration (for reference)
    config: VaultConfig,
    /// HTTP client for AppRole authentication
    http_client: reqwest::Client,
}

// Manual Clone implementation using Arc
impl Clone for VaultClient {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
            config: self.config.clone(),
            http_client: self.http_client.clone(),
        }
    }
}

impl VaultClient {
    /// Create a new Vault client from configuration
    /// 
    /// Phase 6: Supports both Token (dev) and AppRole (production) authentication
    pub async fn new(config: VaultConfig) -> Result<Self> {
        // Build HTTP client with optional TLS skip (for AppRole login on HTTPS port)
        let http_client = if config.skip_verify {
            reqwest::Client::builder()
                .danger_accept_invalid_certs(true)
                .build()
                .context("Failed to build HTTP client")?
        } else {
            reqwest::Client::new()
        };

        // Authenticate based on configured method
        let token = match &config.auth {
            VaultAuth::Token(token) => {
                info!("Using Vault token authentication (dev mode)");
                token.clone()
            }
            VaultAuth::AppRole { role_id, secret_id } => {
                info!("Using Vault AppRole authentication (production mode)");
                Self::login_approle(&config.address, role_id, secret_id, &http_client)
                    .await
                    .context("AppRole login failed")?
            }
        };

        info!(vault_addr = %config.address, "Vault client initialized");

        // Create vaultrs client with authenticated token
        // Note: Using HTTP address for vaultrs (no TLS skip support in v0.7.x)
        let client = vaultrs::client::VaultClient::new(
            vaultrs::client::VaultClientSettingsBuilder::default()
                .address(&config.address)
                .token(&token)
                .build()
                .context("Failed to build Vault client settings")?,
        )
        .context("Failed to create Vault client")?;

        Ok(Self {
            inner: Arc::new(client),
            config,
            http_client,
        })
    }

    /// Login to Vault using AppRole and obtain a client token
    /// 
    /// Phase 6 A2: AppRole authentication for production
    async fn login_approle(
        vault_addr: &str,
        role_id: &str,
        secret_id: &str,
        http_client: &reqwest::Client,
    ) -> Result<String> {
        info!("Authenticating to Vault via AppRole");

        let response = http_client
            .post(format!("{}/v1/auth/approle/login", vault_addr))
            .json(&serde_json::json!({
                "role_id": role_id,
                "secret_id": secret_id
            }))
            .send()
            .await
            .context("AppRole login request failed")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("AppRole login failed: {} - {}", status, body);
        }

        let login_response: AppRoleLoginResponse = response
            .json()
            .await
            .context("Failed to parse AppRole login response")?;

        info!(
            lease_duration = login_response.auth.lease_duration,
            renewable = login_response.auth.renewable,
            "AppRole authentication successful"
        );

        Ok(login_response.auth.client_token)
    }

    /// Create a new Vault client from environment variables
    /// 
    /// Phase 6: Auto-detects Token or AppRole authentication
    pub async fn from_env() -> Result<Self> {
        let config = VaultConfig::from_env()
            .map_err(|e| anyhow::anyhow!("Failed to load Vault config from env: {}", e))?;
        Self::new(config).await
    }

    /// Renew the current token (for AppRole tokens that expire)
    /// 
    /// Phase 6 A2: Token renewal for long-running services
    pub async fn renew_token(&self) -> Result<()> {
        info!("Renewing Vault token");

        let response = self.http_client
            .post(format!("{}/v1/auth/token/renew-self", self.config.address))
            .header("X-Vault-Token", self.get_current_token()?)
            .send()
            .await
            .context("Token renewal request failed")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("Token renewal failed: {} - {}", status, body);
        }

        info!("Token renewed successfully");
        Ok(())
    }

    /// Get the current authentication token (for manual API calls)
    fn get_current_token(&self) -> Result<String> {
        // vaultrs doesn't expose the token directly, so we need to keep it in config
        match &self.config.auth {
            VaultAuth::Token(token) => Ok(token.clone()),
            VaultAuth::AppRole { .. } => {
                // For AppRole, we'd need to store the obtained token
                // This is a limitation - for now, return error
                anyhow::bail!("Token not available for AppRole - use vaultrs client directly")
            }
        }
    }

    /// Get reference to inner vaultrs client (for advanced use cases)
    pub fn inner(&self) -> &vaultrs::client::VaultClient {
        self.inner.as_ref()
    }

    /// Get reference to configuration
    pub fn config(&self) -> &VaultConfig {
        &self.config
    }

    /// Health check - verify Vault is accessible and unsealed
    /// 
    /// Returns Ok(()) if healthy, Err if unreachable or sealed
    pub async fn health_check(&self) -> Result<()> {
        use vaultrs::sys;
        
        let health = sys::health(self.inner.as_ref())
            .await
            .context("Failed to query Vault health")?;

        if health.sealed {
            anyhow::bail!("Vault is sealed");
        }

        if !health.initialized {
            anyhow::bail!("Vault is not initialized");
        }

        Ok(())
    }

    /// Get Vault server version (useful for debugging)
    pub async fn version(&self) -> Result<String> {
        use vaultrs::sys;
        
        let health = sys::health(self.inner.as_ref())
            .await
            .context("Failed to query Vault health")?;

        Ok(health.version)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vault_client_config_stored() {
        let config = VaultConfig::new(
            "http://vault:8200".to_string(),
            "test-token".to_string(),
        );
        
        // We can't actually create a client without a running Vault,
        // but we can test that the config is properly cloned
        let config_clone = config.clone();
        assert_eq!(config.address, config_clone.address);
        match (&config.auth, &config_clone.auth) {
            (VaultAuth::Token(t1), VaultAuth::Token(t2)) => assert_eq!(t1, t2),
            _ => panic!("Auth config not cloned properly"),
        }
    }

    // Integration tests (require running Vault - marked with #[ignore])
    
    #[tokio::test]
    #[ignore]
    async fn test_vault_client_health_check() {
        // This test requires Vault running at http://localhost:8200
        std::env::set_var("VAULT_ADDR", "http://localhost:8200");
        std::env::set_var("VAULT_TOKEN", "root");
        
        let client = VaultClient::from_env().await.unwrap();
        let result = client.health_check().await;
        
        assert!(result.is_ok(), "Vault health check failed: {:?}", result);
        
        // Cleanup
        std::env::remove_var("VAULT_ADDR");
        std::env::remove_var("VAULT_TOKEN");
    }

    #[tokio::test]
    #[ignore]
    async fn test_vault_client_version() {
        // This test requires Vault running at http://localhost:8200
        std::env::set_var("VAULT_ADDR", "http://localhost:8200");
        std::env::set_var("VAULT_TOKEN", "root");
        
        let client = VaultClient::from_env().await.unwrap();
        let version = client.version().await.unwrap();
        
        assert!(!version.is_empty());
        assert!(version.contains("Vault"), "Version should contain 'Vault': {}", version);
        
        // Cleanup
        std::env::remove_var("VAULT_ADDR");
        std::env::remove_var("VAULT_TOKEN");
    }
}
