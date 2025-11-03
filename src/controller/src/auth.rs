use axum::{
    body::Body,
    extract::{Request, State},
    http::{StatusCode, header::AUTHORIZATION},
    middleware::Next,
    response::{IntoResponse, Response},
};
use jsonwebtoken::{decode, decode_header, DecodingKey, Validation, Algorithm};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{error, warn, debug};

/// JWT Claims structure (minimal; extend as needed)
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
    pub iat: Option<usize>,
    pub nbf: Option<usize>,
    pub iss: Option<String>,
    pub aud: Option<serde_json::Value>, // Can be string or array
}

/// JWKS response structure
#[derive(Debug, Deserialize)]
pub struct JwksResponse {
    pub keys: Vec<Jwk>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Jwk {
    pub kty: String,
    pub kid: Option<String>,
    pub n: Option<String>,
    pub e: Option<String>,
    #[serde(rename = "use")]
    pub key_use: Option<String>,
    pub alg: Option<String>,
}

/// JWT configuration
#[derive(Clone)]
pub struct JwtConfig {
    pub jwks_url: String,
    pub issuer: String,
    pub audience: String,
    pub jwks_cache: Arc<tokio::sync::RwLock<Option<JwksResponse>>>,
}

impl JwtConfig {
    pub fn from_env() -> Result<Self, String> {
        let jwks_url = std::env::var("OIDC_JWKS_URL")
            .map_err(|_| "OIDC_JWKS_URL not set".to_string())?;
        let issuer = std::env::var("OIDC_ISSUER_URL")
            .map_err(|_| "OIDC_ISSUER_URL not set".to_string())?;
        let audience = std::env::var("OIDC_AUDIENCE")
            .map_err(|_| "OIDC_AUDIENCE not set".to_string())?;

        Ok(Self {
            jwks_url,
            issuer,
            audience,
            jwks_cache: Arc::new(tokio::sync::RwLock::new(None)),
        })
    }

    /// Fetch JWKS from the provider
    async fn fetch_jwks(&self) -> Result<JwksResponse, String> {
        let client = reqwest::Client::new();
        let response = client
            .get(&self.jwks_url)
            .send()
            .await
            .map_err(|e| format!("Failed to fetch JWKS: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("JWKS endpoint returned {}", response.status()));
        }

        response
            .json::<JwksResponse>()
            .await
            .map_err(|e| format!("Failed to parse JWKS: {}", e))
    }

    /// Get JWKS (from cache or fetch)
    async fn get_jwks(&self) -> Result<JwksResponse, String> {
        // Try cache first
        {
            let cache = self.jwks_cache.read().await;
            if let Some(jwks) = cache.as_ref() {
                debug!("Using cached JWKS");
                return Ok(jwks.clone());
            }
        }

        // Fetch and cache
        let jwks = self.fetch_jwks().await?;
        {
            let mut cache = self.jwks_cache.write().await;
            *cache = Some(jwks.clone());
        }
        debug!("Fetched and cached new JWKS");
        Ok(jwks)
    }

    /// Find key by kid
    fn find_key<'a>(&self, jwks: &'a JwksResponse, kid: &str) -> Option<&'a Jwk> {
        jwks.keys.iter().find(|k| k.kid.as_deref() == Some(kid))
    }

    /// Verify JWT token
    pub async fn verify_token(&self, token: &str) -> Result<Claims, String> {
        // Decode header to get kid
        let header = decode_header(token)
            .map_err(|e| format!("Invalid JWT header: {}", e))?;

        let kid = header.kid.ok_or("Missing kid in JWT header")?;

        // Get JWKS
        let jwks = self.get_jwks().await?;

        // Find the key
        let jwk = self.find_key(&jwks, &kid)
            .ok_or_else(|| format!("Key with kid '{}' not found in JWKS", kid))?;

        // Convert JWK to DecodingKey
        let decoding_key = Self::jwk_to_decoding_key(jwk)?;

        // Setup validation
        let mut validation = Validation::new(Algorithm::RS256);
        validation.set_issuer(&[&self.issuer]);
        validation.set_audience(&[&self.audience]);
        validation.leeway = 60; // 60 seconds clock skew tolerance

        // Decode and validate
        let token_data = decode::<Claims>(token, &decoding_key, &validation)
            .map_err(|e| format!("JWT validation failed: {}", e))?;

        Ok(token_data.claims)
    }

    /// Convert JWK to DecodingKey
    fn jwk_to_decoding_key(jwk: &Jwk) -> Result<DecodingKey, String> {
        if jwk.kty != "RSA" {
            return Err(format!("Unsupported key type: {}", jwk.kty));
        }

        let n = jwk.n.as_ref().ok_or("Missing 'n' in JWK")?;
        let e = jwk.e.as_ref().ok_or("Missing 'e' in JWK")?;

        DecodingKey::from_rsa_components(n, e)
            .map_err(|e| format!("Failed to create DecodingKey: {}", e))
    }
}

/// Extract Bearer token from Authorization header
fn extract_bearer_token(auth_header: &str) -> Option<&str> {
    auth_header.strip_prefix("Bearer ")
}

/// JWT verification middleware
pub async fn jwt_middleware(
    State(config): State<JwtConfig>,
    mut req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Extract Authorization header
    let auth_header = req
        .headers()
        .get(AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .ok_or_else(|| {
            warn!("Missing Authorization header");
            StatusCode::UNAUTHORIZED
        })?;

    // Extract Bearer token
    let token = extract_bearer_token(auth_header).ok_or_else(|| {
        warn!("Invalid Authorization header format (expected 'Bearer <token>')");
        StatusCode::UNAUTHORIZED
    })?;

    // Verify token
    let claims = config.verify_token(token).await.map_err(|e| {
        warn!("JWT verification failed: {}", e);
        StatusCode::UNAUTHORIZED
    })?;

    debug!(
        "JWT verified successfully for subject: {}",
        claims.sub
    );

    // Add claims to request extensions for downstream handlers
    req.extensions_mut().insert(claims);

    Ok(next.run(req).await)
}
