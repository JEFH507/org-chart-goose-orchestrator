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
pub mod verify;  // Phase 6 A5: Profile signature verification

pub use client::VaultClient;
pub use transit::TransitOps;
pub use kv::KvOps;
pub use verify::verify_profile_signature;

/// Vault authentication method
#[derive(Clone, Debug)]
pub enum VaultAuth {
    /// Root token (dev mode only - NOT for production)
    Token(String),
    /// AppRole authentication (production)
    AppRole {
        role_id: String,
        secret_id: String,
    },
}

/// Vault client configuration
#[derive(Clone, Debug)]
pub struct VaultConfig {
    /// Vault server address (e.g., https://vault:8200)
    pub address: String,
    /// Authentication method
    pub auth: VaultAuth,
    /// Transit engine mount path (default: "transit")
    pub transit_mount: String,
    /// KV v2 engine mount path (default: "secret")
    pub kv_mount: String,
    /// Skip TLS verification (dev only)
    pub skip_verify: bool,
}

impl VaultConfig {
    /// Create configuration from environment variables
    /// 
    /// Phase 6: Supports both Token (dev) and AppRole (production) authentication
    /// 
    /// Required env vars:
    /// - VAULT_ADDR: Vault server address
    /// - VAULT_TOKEN: Root token (dev mode) OR
    /// - VAULT_ROLE_ID + VAULT_SECRET_ID: AppRole credentials (production)
    /// 
    /// Optional env vars:
    /// - VAULT_TRANSIT_MOUNT: Transit engine path (default: "transit")
    /// - VAULT_KV_MOUNT: KV v2 engine path (default: "secret")
    /// - VAULT_SKIP_VERIFY: Skip TLS verification for self-signed certs (default: "false")
    pub fn from_env() -> Result<Self, String> {
        let address = std::env::var("VAULT_ADDR")
            .map_err(|_| "VAULT_ADDR environment variable not set".to_string())?;
        
        // Determine authentication method
        let auth = if let (Ok(role_id), Ok(secret_id)) = 
            (std::env::var("VAULT_ROLE_ID"), std::env::var("VAULT_SECRET_ID")) {
            // AppRole authentication (production)
            VaultAuth::AppRole { role_id, secret_id }
        } else if let Ok(token) = std::env::var("VAULT_TOKEN") {
            // Token authentication (dev mode)
            VaultAuth::Token(token)
        } else {
            return Err("Either VAULT_TOKEN or (VAULT_ROLE_ID + VAULT_SECRET_ID) must be set".to_string());
        };
        
        let transit_mount = std::env::var("VAULT_TRANSIT_MOUNT")
            .unwrap_or_else(|_| "transit".to_string());
        let kv_mount = std::env::var("VAULT_KV_MOUNT")
            .unwrap_or_else(|_| "secret".to_string());
        let skip_verify = std::env::var("VAULT_SKIP_VERIFY")
            .unwrap_or_else(|_| "false".to_string())
            .parse::<bool>()
            .unwrap_or(false);

        Ok(Self {
            address,
            auth,
            transit_mount,
            kv_mount,
            skip_verify,
        })
    }

    /// Create configuration with token authentication (dev mode)
    pub fn new(address: String, token: String) -> Self {
        Self {
            address,
            auth: VaultAuth::Token(token),
            transit_mount: "transit".to_string(),
            kv_mount: "secret".to_string(),
            skip_verify: false,
        }
    }

    /// Create configuration with AppRole authentication (production)
    pub fn new_approle(address: String, role_id: String, secret_id: String) -> Self {
        Self {
            address,
            auth: VaultAuth::AppRole { role_id, secret_id },
            transit_mount: "transit".to_string(),
            kv_mount: "secret".to_string(),
            skip_verify: false,
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
        match &config.auth {
            VaultAuth::Token(token) => assert_eq!(token, "test-token"),
            _ => panic!("Expected Token auth"),
        }
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
    fn test_vault_config_from_env_token_success() {
        std::env::set_var("VAULT_ADDR", "http://test:8200");
        std::env::set_var("VAULT_TOKEN", "test-token");
        
        let config = VaultConfig::from_env().unwrap();
        assert_eq!(config.address, "http://test:8200");
        match &config.auth {
            VaultAuth::Token(token) => assert_eq!(token, "test-token"),
            _ => panic!("Expected Token auth"),
        }
        
        // Cleanup
        std::env::remove_var("VAULT_ADDR");
        std::env::remove_var("VAULT_TOKEN");
    }

    #[test]
    fn test_vault_config_from_env_approle_success() {
        std::env::set_var("VAULT_ADDR", "https://test:8200");
        std::env::set_var("VAULT_ROLE_ID", "test-role-id");
        std::env::set_var("VAULT_SECRET_ID", "test-secret-id");
        std::env::set_var("VAULT_SKIP_VERIFY", "true");
        
        let config = VaultConfig::from_env().unwrap();
        assert_eq!(config.address, "https://test:8200");
        assert_eq!(config.skip_verify, true);
        match &config.auth {
            VaultAuth::AppRole { role_id, secret_id } => {
                assert_eq!(role_id, "test-role-id");
                assert_eq!(secret_id, "test-secret-id");
            },
            _ => panic!("Expected AppRole auth"),
        }
        
        // Cleanup
        std::env::remove_var("VAULT_ADDR");
        std::env::remove_var("VAULT_ROLE_ID");
        std::env::remove_var("VAULT_SECRET_ID");
        std::env::remove_var("VAULT_SKIP_VERIFY");
    }
}
