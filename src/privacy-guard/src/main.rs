use axum::{
    extract::{Json, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};

mod detection;
mod pseudonym;
mod redaction;
mod policy;
mod state;
mod audit;
mod ollama_client;

use detection::{detect, detect_hybrid, Rules, EntityType, Detection, Confidence};
use ollama_client::OllamaClient;
use policy::{Policy, GuardMode};
use state::MappingState;
use redaction::{mask, MaskingPolicy};
use audit::log_redaction_event;

// Application state shared across handlers
struct AppState {
    rules: Rules,
    policy: Policy,
    salt: String,
    sessions: RwLock<HashMap<String, Arc<MappingState>>>,
    ollama_client: Arc<OllamaClient>,
}

// Request/Response schemas
#[derive(Deserialize)]
struct ScanRequest {
    text: String,
    #[serde(default)]
    tenant_id: Option<String>,
}

#[derive(Serialize)]
struct ScanResponse {
    detections: Vec<DetectionResponse>,
}

#[derive(Serialize)]
struct DetectionResponse {
    start: usize,
    end: usize,
    entity_type: String,
    confidence: String,
    matched_text: String,
}

#[derive(Deserialize)]
struct MaskRequest {
    text: String,
    tenant_id: String,
    session_id: Option<String>,
    #[serde(default)]
    mode: Option<String>,
    /// Detection method: "rules", "ai", or "hybrid"
    #[serde(default)]
    detection_method: Option<String>,
    /// Privacy mode: "auto", "service-bypass", or "strict"
    #[serde(default)]
    privacy_mode: Option<String>,
}

#[derive(Serialize)]
struct MaskResponse {
    masked_text: String,
    redactions: HashMap<String, usize>,
    session_id: String,
}

#[derive(Deserialize)]
struct ReidentifyRequest {
    pseudonym: String,
    session_id: String,
}

#[derive(Serialize)]
struct ReidentifyResponse {
    original: String,
}

#[derive(Deserialize)]
struct FlushSessionRequest {
    session_id: String,
}

#[derive(Serialize)]
struct FlushSessionResponse {
    status: String,
}

#[derive(Serialize)]
struct StatusResponse {
    status: String,
    mode: String,
    rule_count: usize,
    config_loaded: bool,
    model_enabled: bool,
    model_name: String,
}

// Error types
#[derive(Debug)]
enum AppError {
    InvalidInput(String),
    Unauthorized,
    NotFound,
    InvalidMode,
    Internal(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::InvalidInput(msg) => (StatusCode::BAD_REQUEST, msg),
            AppError::Unauthorized => (StatusCode::UNAUTHORIZED, "Unauthorized".to_string()),
            AppError::NotFound => (StatusCode::NOT_FOUND, "Not found".to_string()),
            AppError::InvalidMode => (StatusCode::BAD_REQUEST, "Invalid mode".to_string()),
            AppError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
        };
        (status, Json(serde_json::json!({"error": message}))).into_response()
    }
}

impl From<String> for AppError {
    fn from(s: String) -> Self {
        AppError::Internal(s)
    }
}

impl From<&str> for AppError {
    fn from(s: &str) -> Self {
        AppError::Internal(s.to_string())
    }
}

// Handlers
async fn status_handler(State(state): State<Arc<AppState>>) -> Json<StatusResponse> {
    Json(StatusResponse {
        status: "healthy".to_string(),
        mode: format!("{:?}", state.policy.mode),
        rule_count: state.rules.count(),
        config_loaded: true,
        model_enabled: state.ollama_client.is_enabled(),
        model_name: state.ollama_client.model_name().to_string(),
    })
}

async fn scan_handler(
    State(state): State<Arc<AppState>>,
    Json(req): Json<ScanRequest>,
) -> Result<Json<ScanResponse>, AppError> {
    info!(
        tenant_id = req.tenant_id.as_deref().unwrap_or("unknown"),
        text_length = req.text.len(),
        "Received scan request"
    );

    // Use hybrid detection (regex + model)
    let detections = detect_hybrid(&req.text, &state.rules, &state.ollama_client).await;
    
    let response_detections = detections
        .into_iter()
        .map(|d| DetectionResponse {
            start: d.start,
            end: d.end,
            entity_type: format!("{:?}", d.entity_type),
            confidence: format!("{:?}", d.confidence),
            matched_text: d.matched_text,
        })
        .collect();

    Ok(Json(ScanResponse {
        detections: response_detections,
    }))
}

async fn mask_handler(
    State(state): State<Arc<AppState>>,
    Json(req): Json<MaskRequest>,
) -> Result<Json<MaskResponse>, AppError> {
    let start_time = std::time::Instant::now();
    
    // Parse detection method from request (default to "hybrid")
    let detection_method = req.detection_method.as_deref().unwrap_or("hybrid");
    
    // Parse privacy mode from request (default to "auto")
    let privacy_mode = req.privacy_mode.as_deref().unwrap_or("auto");
    
    info!(
        tenant_id = %req.tenant_id,
        text_length = req.text.len(),
        detection_method = detection_method,
        privacy_mode = privacy_mode,
        "Received mask request with user settings"
    );

    // Validate tenant_id
    if req.tenant_id.is_empty() {
        return Err(AppError::InvalidInput("tenant_id is required".to_string()));
    }

    // Generate or use provided session_id
    let session_id = req.session_id.unwrap_or_else(|| {
        format!("sess_{}", uuid::Uuid::new_v4())
    });

    // Get or create session state
    let session_state = {
        let mut sessions = state.sessions.write().await;
        sessions
            .entry(session_id.clone())
            .or_insert_with(|| Arc::new(MappingState::new()))
            .clone()
    };

    // Handle privacy_mode = "service-bypass" (no masking, just audit)
    if privacy_mode == "service-bypass" {
        info!(
            tenant_id = %req.tenant_id,
            "Privacy mode: SERVICE-BYPASS - Skipping masking (audit only)"
        );
        
        // Log audit event with zero redactions
        let duration_ms = start_time.elapsed().as_millis() as u64;
        log_redaction_event(
            &req.tenant_id,
            Some(&session_id),
            state.policy.mode,
            &HashMap::new(),
            duration_ms,
        );
        
        return Ok(Json(MaskResponse {
            masked_text: req.text.clone(),
            redactions: HashMap::new(),
            session_id,
        }));
    }

    // Check if policy allows masking (legacy check)
    if !state.policy.should_mask() {
        // If not in MASK mode, just detect (using appropriate method)
        let detections = match detection_method {
            "rules" => detect(&req.text, &state.rules),
            "ai" | _ => detect_hybrid(&req.text, &state.rules, &state.ollama_client).await,
        };
        let filtered = state.policy.filter_detections(detections);
        
        // Return unmasked text with empty redactions
        return Ok(Json(MaskResponse {
            masked_text: req.text.clone(),
            redactions: HashMap::new(),
            session_id,
        }));
    }

    // Check if PSEUDO_SALT is available for MASK mode
    if state.salt.is_empty() {
        warn!("PSEUDO_SALT not set, cannot mask in MASK mode");
        return Err(AppError::Internal(
            "PSEUDO_SALT not configured, masking unavailable".to_string(),
        ));
    }

    // Step 1: Detect PII using user-selected detection method
    let detections = match detection_method {
        "rules" => {
            info!("Using rules-only detection (fast ~10ms)");
            detect(&req.text, &state.rules)
        }
        "ai" | _ => {
            // For "ai" mode, we use hybrid which will use the model if available
            // This way we don't need to handle Vec<NerEntity> vs Vec<Detection> conversion
            info!("Using hybrid/AI detection (balanced ~100ms or accurate ~15s)");
            detect_hybrid(&req.text, &state.rules, &state.ollama_client).await
        }
    };

    // Step 2: Filter by confidence threshold
    let filtered_detections = state.policy.filter_detections(detections);

    // Step 3: Apply masking
    let mask_result = mask(
        &req.text,
        filtered_detections,
        &state.policy.masking_policy,
        &*session_state,
        &req.tenant_id,
    );

    // Log masked text for debugging (verbatim payload sent to LLM)
    info!(
        session_id = %session_id,
        original_length = req.text.len(),
        masked_length = mask_result.masked_text.len(),
        redactions = ?mask_result.redactions,
        "Masked payload: {}",
        mask_result.masked_text
    );

    // Log audit event
    let duration_ms = start_time.elapsed().as_millis() as u64;
    log_redaction_event(
        &req.tenant_id,
        Some(&session_id),
        state.policy.mode,
        &mask_result.redactions,
        duration_ms,
    );

    Ok(Json(MaskResponse {
        masked_text: mask_result.masked_text,
        redactions: mask_result.redactions,
        session_id,
    }))
}

async fn reidentify_handler(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    Json(req): Json<ReidentifyRequest>,
) -> Result<Json<ReidentifyResponse>, AppError> {
    // Validate JWT from Authorization header
    if !validate_jwt(&headers) {
        warn!("Unauthorized reidentify request");
        return Err(AppError::Unauthorized);
    }

    info!(
        session_id = %req.session_id,
        "Received reidentify request"
    );

    // Get session state
    let sessions = state.sessions.read().await;
    let session_state = sessions
        .get(&req.session_id)
        .ok_or(AppError::NotFound)?;

    // Lookup original value
    let original = session_state
        .get_original(&req.pseudonym)
        .ok_or(AppError::NotFound)?;

    Ok(Json(ReidentifyResponse { original }))
}

async fn flush_session_handler(
    State(state): State<Arc<AppState>>,
    Json(req): Json<FlushSessionRequest>,
) -> Result<Json<FlushSessionResponse>, AppError> {
    info!(session_id = %req.session_id, "Flushing session");

    let mut sessions = state.sessions.write().await;
    if let Some(session) = sessions.remove(&req.session_id) {
        session.clear();
        Ok(Json(FlushSessionResponse {
            status: "flushed".to_string(),
        }))
    } else {
        Err(AppError::NotFound)
    }
}

// Helper functions
fn validate_jwt(headers: &HeaderMap) -> bool {
    // Extract Authorization header
    let auth_header = match headers.get("authorization") {
        Some(h) => h.to_str().unwrap_or(""),
        None => return false,
    };

    // Check for Bearer token
    if !auth_header.starts_with("Bearer ") {
        return false;
    }

    // TODO: Implement full JWT validation with RS256 and JWKS
    // For now, just check that a token is present
    let token = &auth_header[7..];
    !token.is_empty()
}

fn derive_fpe_key(salt: &str) -> Vec<u8> {
    use sha2::{Sha256, Digest};
    
    let mut hasher = Sha256::new();
    hasher.update(salt.as_bytes());
    hasher.finalize().to_vec()
}

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    // Load configuration
    let salt = std::env::var("PSEUDO_SALT").unwrap_or_else(|_| {
        warn!("PSEUDO_SALT not set, masking will be unavailable");
        String::new()
    });

    let rules = Rules::default_rules();
    let policy = Policy::default();
    
    // Initialize Ollama client
    let ollama_client = Arc::new(OllamaClient::from_env());
    
    // Check Ollama health (non-blocking)
    let ollama_healthy = ollama_client.health_check().await;
    if ollama_client.is_enabled() && !ollama_healthy {
        warn!("Ollama health check failed, model detection will fall back to regex-only");
    }

    info!(
        mode = ?policy.mode,
        rule_count = rules.count(),
        salt_configured = !salt.is_empty(),
        model_enabled = ollama_client.is_enabled(),
        model_name = ollama_client.model_name(),
        "Privacy Guard starting"
    );

    let app_state = Arc::new(AppState {
        rules,
        policy,
        salt,
        sessions: RwLock::new(HashMap::new()),
        ollama_client,
    });

    // Build router
    let app = Router::new()
        .route("/status", get(status_handler))
        .route("/guard/scan", post(scan_handler))
        .route("/guard/mask", post(mask_handler))
        .route("/guard/reidentify", post(reidentify_handler))
        .route("/internal/flush-session", post(flush_session_handler))
        .with_state(app_state);

    // Get port from environment or use default
    let port = std::env::var("GUARD_PORT").unwrap_or_else(|_| "8089".to_string());
    let addr = format!("0.0.0.0:{}", port);

    info!("Privacy Guard listening on {}", addr);

    // Start server
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_status_endpoint() {
        let app_state = Arc::new(AppState {
            rules: Rules::default_rules(),
            policy: Policy::default(),
            salt: "test-salt".to_string(),
            sessions: RwLock::new(HashMap::new()),
            ollama_client: Arc::new(OllamaClient::new(
                "http://localhost:11434".to_string(),
                "qwen3:0.6b".to_string(),
                false,
            )),
        });

        let app = Router::new()
            .route("/status", get(status_handler))
            .with_state(app_state);

        let response = app
            .oneshot(Request::builder().uri("/status").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_scan_endpoint() {
        let app_state = Arc::new(AppState {
            rules: Rules::default_rules(),
            policy: Policy::default(),
            salt: "test-salt".to_string(),
            sessions: RwLock::new(HashMap::new()),
            ollama_client: Arc::new(OllamaClient::new(
                "http://localhost:11434".to_string(),
                "qwen3:0.6b".to_string(),
                false,
            )),
        });

        let app = Router::new()
            .route("/guard/scan", post(scan_handler))
            .with_state(app_state);

        let body = serde_json::json!({
            "text": "Contact john@example.com",
            "tenant_id": "test-org"
        });

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/guard/scan")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_string(&body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_mask_endpoint() {
        let app_state = Arc::new(AppState {
            rules: Rules::default_rules(),
            policy: Policy::default(),
            salt: "test-salt-for-hmac".to_string(),
            sessions: RwLock::new(HashMap::new()),
            ollama_client: Arc::new(OllamaClient::new(
                "http://localhost:11434".to_string(),
                "qwen3:0.6b".to_string(),
                false,
            )),
        });

        let app = Router::new()
            .route("/guard/mask", post(mask_handler))
            .with_state(app_state);

        let body = serde_json::json!({
            "text": "Contact john@example.com",
            "tenant_id": "test-org"
        });

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/guard/mask")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_string(&body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_reidentify_unauthorized() {
        let app_state = Arc::new(AppState {
            rules: Rules::default_rules(),
            policy: Policy::default(),
            salt: "test-salt".to_string(),
            sessions: RwLock::new(HashMap::new()),
            ollama_client: Arc::new(OllamaClient::new(
                "http://localhost:11434".to_string(),
                "qwen3:0.6b".to_string(),
                false,
            )),
        });

        let app = Router::new()
            .route("/guard/reidentify", post(reidentify_handler))
            .with_state(app_state);

        let body = serde_json::json!({
            "pseudonym": "EMAIL_abc123",
            "session_id": "sess_test"
        });

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/guard/reidentify")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_string(&body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_flush_session() {
        let app_state = Arc::new(AppState {
            rules: Rules::default_rules(),
            policy: Policy::default(),
            salt: "test-salt".to_string(),
            sessions: RwLock::new(HashMap::new()),
            ollama_client: Arc::new(OllamaClient::new(
                "http://localhost:11434".to_string(),
                "qwen3:0.6b".to_string(),
                false,
            )),
        });

        // Add a session first
        {
            let mut sessions = app_state.sessions.write().await;
            sessions.insert(
                "sess_test".to_string(),
                Arc::new(MappingState::new()),
            );
        }

        let app = Router::new()
            .route("/internal/flush-session", post(flush_session_handler))
            .with_state(app_state);

        let body = serde_json::json!({
            "session_id": "sess_test"
        });

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/internal/flush-session")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_string(&body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }
}
