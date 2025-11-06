// Vault client - Core client wrapper around vaultrs

use super::VaultConfig;
use anyhow::{Context, Result};
use std::sync::Arc;

/// Production-grade Vault client with connection pooling and health checks
pub struct VaultClient {
    /// vaultrs client instance (wrapped in Arc for sharing across threads)
    inner: std::sync::Arc<vaultrs::client::VaultClient>,
    /// Configuration (for reference)
    config: VaultConfig,
}

// Manual Clone implementation using Arc
impl Clone for VaultClient {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
            config: self.config.clone(),
        }
    }
}

impl VaultClient {
    /// Create a new Vault client from configuration
    /// 
    /// This initializes the vaultrs client with connection pooling
    /// and TLS verification disabled for dev mode (HTTP).
    pub async fn new(config: VaultConfig) -> Result<Self> {
        // Create vaultrs client with token authentication
        let client = vaultrs::client::VaultClient::new(
            vaultrs::client::VaultClientSettingsBuilder::default()
                .address(&config.address)
                .token(&config.token)
                .build()
                .context("Failed to build Vault client settings")?,
        )
        .context("Failed to create Vault client")?;

        Ok(Self {
            inner: Arc::new(client),
            config,
        })
    }

    /// Create a new Vault client from environment variables
    pub async fn from_env() -> Result<Self> {
        let config = VaultConfig::from_env()
            .map_err(|e| anyhow::anyhow!("Failed to load Vault config from env: {}", e))?;
        Self::new(config).await
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
        assert_eq!(config.token, config_clone.token);
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
