// Phase 5: Vault Client - Production-grade HashiCorp Vault integration
//
// This module provides a centralized, production-ready Vault client for:
// - Phase 5: Profile HMAC signing (Transit engine)
// - Phase 6: Privacy Guard PII redaction rules (KV v2 engine)
// - Phase 7+: Secrets management, PKI, dynamic database credentials
//
// Architecture:
// - Based on `vaultrs` 0.7.x (official Rust client)
// - Connection pooling via reqwest
// - Automatic token renewal (future enhancement)
// - Health checks and retry logic
// - Support for multiple Vault engines (Transit, KV v2, Database, PKI)

pub mod client;
pub mod transit;
pub mod kv;

pub use client::VaultClient;
pub use transit::TransitOps;
pub use kv::KvOps;

/// Vault client configuration
#[derive(Clone, Debug)]
pub struct VaultConfig {
    /// Vault server address (e.g., http://vault:8200)
    pub address: String,
    /// Vault authentication token
    pub token: String,
    /// Transit engine mount path (default: "transit")
    pub transit_mount: String,
    /// KV v2 engine mount path (default: "secret")
    pub kv_mount: String,
}

impl VaultConfig {
    /// Create configuration from environment variables
    /// 
    /// Required env vars:
    /// - VAULT_ADDR: Vault server address
    /// - VAULT_TOKEN: Vault authentication token
    /// 
    /// Optional env vars:
    /// - VAULT_TRANSIT_MOUNT: Transit engine path (default: "transit")
    /// - VAULT_KV_MOUNT: KV v2 engine path (default: "secret")
    pub fn from_env() -> Result<Self, String> {
        let address = std::env::var("VAULT_ADDR")
            .map_err(|_| "VAULT_ADDR environment variable not set".to_string())?;
        let token = std::env::var("VAULT_TOKEN")
            .map_err(|_| "VAULT_TOKEN environment variable not set".to_string())?;
        
        let transit_mount = std::env::var("VAULT_TRANSIT_MOUNT")
            .unwrap_or_else(|_| "transit".to_string());
        let kv_mount = std::env::var("VAULT_KV_MOUNT")
            .unwrap_or_else(|_| "secret".to_string());

        Ok(Self {
            address,
            token,
            transit_mount,
            kv_mount,
        })
    }

    /// Create configuration with custom values
    pub fn new(address: String, token: String) -> Self {
        Self {
            address,
            token,
            transit_mount: "transit".to_string(),
            kv_mount: "secret".to_string(),
        }
    }

    /// Set custom Transit engine mount path
    pub fn with_transit_mount(mut self, mount: String) -> Self {
        self.transit_mount = mount;
        self
    }

    /// Set custom KV v2 engine mount path
    pub fn with_kv_mount(mut self, mount: String) -> Self {
        self.kv_mount = mount;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vault_config_defaults() {
        let config = VaultConfig::new(
            "http://vault:8200".to_string(),
            "test-token".to_string(),
        );
        
        assert_eq!(config.address, "http://vault:8200");
        assert_eq!(config.token, "test-token");
        assert_eq!(config.transit_mount, "transit");
        assert_eq!(config.kv_mount, "secret");
    }

    #[test]
    fn test_vault_config_custom_mounts() {
        let config = VaultConfig::new(
            "http://vault:8200".to_string(),
            "test-token".to_string(),
        )
        .with_transit_mount("custom-transit".to_string())
        .with_kv_mount("custom-kv".to_string());
        
        assert_eq!(config.transit_mount, "custom-transit");
        assert_eq!(config.kv_mount, "custom-kv");
    }

    #[test]
    fn test_vault_config_from_env_missing() {
        std::env::remove_var("VAULT_ADDR");
        std::env::remove_var("VAULT_TOKEN");
        
        let result = VaultConfig::from_env();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("VAULT_ADDR"));
    }

    #[test]
    fn test_vault_config_from_env_success() {
        std::env::set_var("VAULT_ADDR", "http://test:8200");
        std::env::set_var("VAULT_TOKEN", "test-token");
        
        let config = VaultConfig::from_env().unwrap();
        assert_eq!(config.address, "http://test:8200");
        assert_eq!(config.token, "test-token");
        
        // Cleanup
        std::env::remove_var("VAULT_ADDR");
        std::env::remove_var("VAULT_TOKEN");
    }
}
