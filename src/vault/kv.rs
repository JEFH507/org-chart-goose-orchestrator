// Vault KV v2 Engine Operations - For Privacy Guard PII redaction rules (Phase 6)

use super::VaultClient;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// KV v2 engine operations for secret storage and retrieval
pub struct KvOps {
    client: VaultClient,
}

impl KvOps {
    /// Create a new KvOps instance
    pub fn new(client: VaultClient) -> Self {
        Self { client }
    }

    /// Read a secret from KV v2 store
    /// 
    /// Args:
    /// - path: Secret path (e.g., "privacy/pii-rules/finance")
    /// 
    /// Returns: Secret data as HashMap
    pub async fn read(&self, path: &str) -> Result<HashMap<String, String>> {
        let secret = vaultrs::kv2::read::<HashMap<String, String>>(
            self.client.inner(),
            &self.client.config().kv_mount,
            path,
        )
        .await
        .context(format!("Failed to read secret at path: {}", path))?;

        Ok(secret)
    }

    /// Write a secret to KV v2 store
    /// 
    /// Args:
    /// - path: Secret path
    /// - data: Secret data
    /// 
    /// Returns: Version number of created secret
    pub async fn write(&self, path: &str, data: &HashMap<String, String>) -> Result<u64> {
        let response = vaultrs::kv2::set(
            self.client.inner(),
            &self.client.config().kv_mount,
            path,
            data,
        )
        .await
        .context(format!("Failed to write secret at path: {}", path))?;

        Ok(response.version)
    }

    /// Delete a secret from KV v2 store (soft delete)
    /// 
    /// This marks the latest version as deleted but preserves it for recovery.
    /// Use destroy_versions() for permanent deletion.
    pub async fn delete(&self, path: &str) -> Result<()> {
        vaultrs::kv2::delete_latest(
            self.client.inner(),
            &self.client.config().kv_mount,
            path,
        )
        .await
        .map_err(|e| anyhow::anyhow!("Failed to delete secret at path {}: {}", path, e))?;

        Ok(())
    }

    /// List secrets at a path (non-recursive)
    /// 
    /// Returns: List of secret keys
    pub async fn list(&self, path: &str) -> Result<Vec<String>> {
        let keys = vaultrs::kv2::list(
            self.client.inner(),
            &self.client.config().kv_mount,
            path,
        )
        .await
        .context(format!("Failed to list secrets at path: {}", path))?;

        Ok(keys)
    }
}

/// Phase 6: PII Redaction Rule (stored in Vault KV v2)
/// 
/// This will be used by Privacy Guard MCP to dynamically load
/// redaction rules from Vault instead of hardcoding them.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PiiRedactionRule {
    /// Rule name (e.g., "ssn", "credit-card", "email")
    pub name: String,
    /// Regex pattern to match PII
    pub pattern: String,
    /// Replacement template (e.g., "[REDACTED-SSN]", "***-**-{last4}")
    pub replacement: String,
    /// PII category (e.g., "financial", "personal", "medical")
    pub category: String,
    /// Whether this rule is enabled
    pub enabled: bool,
}

impl PiiRedactionRule {
    /// Serialize to HashMap for Vault storage
    pub fn to_vault_map(&self) -> HashMap<String, String> {
        let mut map = HashMap::new();
        map.insert("name".to_string(), self.name.clone());
        map.insert("pattern".to_string(), self.pattern.clone());
        map.insert("replacement".to_string(), self.replacement.clone());
        map.insert("category".to_string(), self.category.clone());
        map.insert("enabled".to_string(), self.enabled.to_string());
        map
    }

    /// Deserialize from Vault HashMap
    pub fn from_vault_map(map: HashMap<String, String>) -> Result<Self> {
        Ok(Self {
            name: map.get("name")
                .context("Missing 'name' field")?
                .clone(),
            pattern: map.get("pattern")
                .context("Missing 'pattern' field")?
                .clone(),
            replacement: map.get("replacement")
                .context("Missing 'replacement' field")?
                .clone(),
            category: map.get("category")
                .context("Missing 'category' field")?
                .clone(),
            enabled: map.get("enabled")
                .context("Missing 'enabled' field")?
                .parse()
                .context("Invalid boolean for 'enabled'")?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pii_rule_serialization() {
        let rule = PiiRedactionRule {
            name: "ssn".to_string(),
            pattern: r"\b\d{3}-\d{2}-\d{4}\b".to_string(),
            replacement: "[REDACTED-SSN]".to_string(),
            category: "financial".to_string(),
            enabled: true,
        };
        
        let map = rule.to_vault_map();
        assert_eq!(map.get("name").unwrap(), "ssn");
        assert_eq!(map.get("enabled").unwrap(), "true");
        
        let deserialized = PiiRedactionRule::from_vault_map(map).unwrap();
        assert_eq!(rule, deserialized);
    }

    #[test]
    fn test_pii_rule_from_vault_map_missing_field() {
        let mut map = HashMap::new();
        map.insert("name".to_string(), "test".to_string());
        // Missing other required fields
        
        let result = PiiRedactionRule::from_vault_map(map);
        assert!(result.is_err());
    }

    // Integration tests (require running Vault with KV v2 engine)
    
    #[tokio::test]
    #[ignore]
    async fn test_kv_read_write() {
        std::env::set_var("VAULT_ADDR", "http://localhost:8200");
        std::env::set_var("VAULT_TOKEN", "root");
        
        let client = VaultClient::from_env().await.unwrap();
        let kv = KvOps::new(client);
        
        // Write a secret
        let mut data = HashMap::new();
        data.insert("key1".to_string(), "value1".to_string());
        data.insert("key2".to_string(), "value2".to_string());
        
        let version = kv.write("test/secret", &data).await.unwrap();
        assert!(version > 0);
        
        // Read it back
        let read_data = kv.read("test/secret").await.unwrap();
        assert_eq!(read_data.get("key1").unwrap(), "value1");
        assert_eq!(read_data.get("key2").unwrap(), "value2");
        
        // Delete it
        kv.delete("test/secret").await.unwrap();
        
        // Cleanup
        std::env::remove_var("VAULT_ADDR");
        std::env::remove_var("VAULT_TOKEN");
    }

    #[tokio::test]
    #[ignore]
    async fn test_kv_list() {
        std::env::set_var("VAULT_ADDR", "http://localhost:8200");
        std::env::set_var("VAULT_TOKEN", "root");
        
        let client = VaultClient::from_env().await.unwrap();
        let kv = KvOps::new(client);
        
        // Write multiple secrets
        let data = HashMap::new();
        kv.write("test/list/secret1", &data).await.unwrap();
        kv.write("test/list/secret2", &data).await.unwrap();
        
        // List them
        let keys = kv.list("test/list").await.unwrap();
        assert!(keys.contains(&"secret1".to_string()));
        assert!(keys.contains(&"secret2".to_string()));
        
        // Cleanup
        kv.delete("test/list/secret1").await.unwrap();
        kv.delete("test/list/secret2").await.unwrap();
        std::env::remove_var("VAULT_ADDR");
        std::env::remove_var("VAULT_TOKEN");
    }

    #[tokio::test]
    #[ignore]
    async fn test_kv_pii_rule_storage() {
        std::env::set_var("VAULT_ADDR", "http://localhost:8200");
        std::env::set_var("VAULT_TOKEN", "root");
        
        let client = VaultClient::from_env().await.unwrap();
        let kv = KvOps::new(client);
        
        // Create a PII rule
        let rule = PiiRedactionRule {
            name: "email".to_string(),
            pattern: r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b".to_string(),
            replacement: "[REDACTED-EMAIL]".to_string(),
            category: "personal".to_string(),
            enabled: true,
        };
        
        // Store in Vault
        let data = rule.to_vault_map();
        kv.write("privacy/rules/email", &data).await.unwrap();
        
        // Read back
        let read_data = kv.read("privacy/rules/email").await.unwrap();
        let read_rule = PiiRedactionRule::from_vault_map(read_data).unwrap();
        
        assert_eq!(rule, read_rule);
        
        // Cleanup
        kv.delete("privacy/rules/email").await.unwrap();
        std::env::remove_var("VAULT_ADDR");
        std::env::remove_var("VAULT_TOKEN");
    }
}
