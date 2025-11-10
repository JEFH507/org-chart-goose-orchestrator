use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Privacy modes for the proxy
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum PrivacyMode {
    /// Auto mode: mask text content, bypass binary with warning
    Auto,
    /// Bypass mode: no masking, all requests logged
    Bypass,
    /// Strict mode: error on PII or unsupported content types
    Strict,
}

/// Detection methods for PII scanning
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum DetectionMethod {
    /// Rules-based regex patterns only (fast)
    Rules,
    /// AI model (Ollama NER) only (slower, more accurate)
    Ner,
    /// Both rules and NER (recommended, most comprehensive)
    Hybrid,
}

impl Default for PrivacyMode {
    fn default() -> Self {
        PrivacyMode::Auto
    }
}

impl std::fmt::Display for PrivacyMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PrivacyMode::Auto => write!(f, "auto"),
            PrivacyMode::Bypass => write!(f, "bypass"),
            PrivacyMode::Strict => write!(f, "strict"),
        }
    }
}

impl Default for DetectionMethod {
    fn default() -> Self {
        DetectionMethod::Hybrid
    }
}

impl std::fmt::Display for DetectionMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DetectionMethod::Rules => write!(f, "rules"),
            DetectionMethod::Ner => write!(f, "ner"),
            DetectionMethod::Hybrid => write!(f, "hybrid"),
        }
    }
}

/// Activity log entry for tracking proxy operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityLogEntry {
    pub timestamp: DateTime<Utc>,
    pub action: String,
    pub content_type: String,
    pub details: String,
}

impl ActivityLogEntry {
    pub fn new(action: impl Into<String>, content_type: impl Into<String>, details: impl Into<String>) -> Self {
        Self {
            timestamp: Utc::now(),
            action: action.into(),
            content_type: content_type.into(),
            details: details.into(),
        }
    }
}

/// Shared state for the proxy service
#[derive(Clone)]
pub struct ProxyState {
    pub current_mode: Arc<RwLock<PrivacyMode>>,
    pub detection_method: Arc<RwLock<DetectionMethod>>,
    pub allow_override: Arc<RwLock<bool>>,
    pub activity_log: Arc<RwLock<Vec<ActivityLogEntry>>>,
    pub privacy_guard_url: String,
}

impl ProxyState {
    pub fn new(privacy_guard_url: String) -> Self {
        Self {
            current_mode: Arc::new(RwLock::new(PrivacyMode::default())),
            detection_method: Arc::new(RwLock::new(DetectionMethod::default())),
            allow_override: Arc::new(RwLock::new(true)), // Default: allow user control
            activity_log: Arc::new(RwLock::new(Vec::new())),
            privacy_guard_url,
        }
    }

    /// Get the current privacy mode
    pub async fn get_mode(&self) -> PrivacyMode {
        *self.current_mode.read().await
    }

    /// Set the privacy mode
    pub async fn set_mode(&self, mode: PrivacyMode) {
        let mut current = self.current_mode.write().await;
        *current = mode;
        
        // Log the mode change
        self.log_activity(
            "mode_change",
            "system",
            format!("Privacy mode changed to: {}", mode),
        ).await;
    }

    /// Log an activity entry
    pub async fn log_activity(
        &self,
        action: impl Into<String>,
        content_type: impl Into<String>,
        details: impl Into<String>,
    ) {
        let mut log = self.activity_log.write().await;
        log.push(ActivityLogEntry::new(action, content_type, details));
        
        // Keep only last 100 entries to prevent unbounded growth
        if log.len() > 100 {
            let excess = log.len() - 100;
            log.drain(0..excess);
        }
    }

    /// Get recent activity entries (last N entries)
    pub async fn get_recent_activity(&self, limit: usize) -> Vec<ActivityLogEntry> {
        let log = self.activity_log.read().await;
        let start = if log.len() > limit {
            log.len() - limit
        } else {
            0
        };
        log[start..].to_vec()
    }

    /// Get total activity count
    pub async fn get_activity_count(&self) -> usize {
        self.activity_log.read().await.len()
    }

    /// Get the current detection method
    pub async fn get_detection_method(&self) -> DetectionMethod {
        *self.detection_method.read().await
    }

    /// Set the detection method (only if override allowed)
    pub async fn set_detection_method(&self, method: DetectionMethod) -> Result<(), String> {
        let allow_override = *self.allow_override.read().await;
        
        if !allow_override {
            return Err("Detection method is locked by profile configuration".to_string());
        }
        
        let mut current = self.detection_method.write().await;
        *current = method;
        
        // Log the change
        self.log_activity(
            "detection_method_change",
            "system",
            format!("Detection method changed to: {}", method),
        ).await;
        
        Ok(())
    }

    /// Get whether override is allowed
    pub async fn get_allow_override(&self) -> bool {
        *self.allow_override.read().await
    }

    /// Set whether override is allowed (admin-only, set from profile)
    pub async fn set_allow_override(&self, allowed: bool) {
        let mut current = self.allow_override.write().await;
        *current = allowed;
    }
}
