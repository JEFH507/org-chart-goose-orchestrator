// Phase 4: Idempotency middleware for POST requests
//
// Implements idempotency key deduplication using Redis as cache.
//
// Flow:
// 1. Extract Idempotency-Key header from request
// 2. Check Redis: GET idempotency:{key}
//    - If exists: return cached response (HTTP 200, same body as original)
//    - If not exists: process request, cache response (SET with TTL)
// 3. TTL: 24 hours (configurable via IDEMPOTENCY_TTL_SECONDS env var)
//
// Applies to: POST /tasks/route, POST /approvals, POST /sessions
//
// Missing Idempotency-Key header: Process as new request (don't cache)

use axum::{
    body::Body,
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::{info, warn};

use crate::AppState;

const IDEMPOTENCY_KEY_HEADER: &str = "idempotency-key";
const DEFAULT_TTL_SECONDS: u64 = 86400; // 24 hours

#[derive(Serialize, Deserialize)]
struct CachedResponse {
    status: u16,
    body: String,
    headers: Vec<(String, String)>,
}

/// Idempotency middleware for POST requests
///
/// Checks Redis cache for duplicate idempotency keys and returns cached responses.
/// If key is new, processes request and caches the response.
pub async fn idempotency_middleware(
    State(state): State<AppState>,
    headers: HeaderMap,
    request: Request,
    next: Next,
) -> Response {
    // Only process if Redis is configured and idempotency is enabled
    let mut redis = match state.redis_client.clone() {
        Some(client) => client,
        None => {
            // Redis not configured, pass through
            return next.run(request).await;
        }
    };

    // Extract idempotency key from header
    let idempotency_key = match headers.get(IDEMPOTENCY_KEY_HEADER) {
        Some(key) => match key.to_str() {
            Ok(k) => k.to_string(),
            Err(_) => {
                warn!(message = "invalid idempotency-key header");
                return next.run(request).await;
            }
        },
        None => {
            // No idempotency key, pass through (don't cache)
            return next.run(request).await;
        }
    };

    let cache_key = format!("idempotency:{}", idempotency_key);

    // Check Redis cache
    match redis.get::<_, Option<String>>(&cache_key).await {
        Ok(Some(cached_json)) => {
            // Cache hit: return cached response
            match serde_json::from_str::<CachedResponse>(&cached_json) {
                Ok(cached) => {
                    info!(
                        message = "idempotency cache hit",
                        key = %idempotency_key,
                        status = cached.status
                    );

                    let mut response = Response::builder()
                        .status(StatusCode::from_u16(cached.status).unwrap_or(StatusCode::OK));

                    // Restore headers
                    for (name, value) in cached.headers {
                        if let Ok(header_name) = name.parse::<axum::http::HeaderName>() {
                            if let Ok(header_value) = value.parse::<axum::http::HeaderValue>() {
                                response = response.header(header_name, header_value);
                            }
                        }
                    }

                    return response
                        .body(Body::from(cached.body))
                        .unwrap()
                        .into_response();
                }
                Err(e) => {
                    warn!(
                        message = "failed to deserialize cached response",
                        error = %e,
                        key = %idempotency_key
                    );
                    // Fall through to process request
                }
            }
        }
        Ok(None) => {
            // Cache miss: process request
        }
        Err(e) => {
            warn!(
                message = "redis get error",
                error = %e,
                key = %idempotency_key
            );
            // Fail open: process request even if Redis fails
        }
    }

    // Process request
    let response = next.run(request).await;

    // Cache response if status is 2xx (success) or 4xx (client error, idempotent)
    let status = response.status();
    if status.is_success() || status.is_client_error() {
        // Extract response parts for caching
        let (parts, body) = response.into_parts();
        
        // Convert body to bytes
        let body_bytes = match axum::body::to_bytes(body, usize::MAX).await {
            Ok(bytes) => bytes,
            Err(e) => {
                warn!(message = "failed to read response body", error = %e);
                // Return original response without caching
                return Response::from_parts(parts, Body::empty());
            }
        };

        let body_string = String::from_utf8_lossy(&body_bytes).to_string();

        // Extract headers for caching
        let headers_vec: Vec<(String, String)> = parts
            .headers
            .iter()
            .filter_map(|(name, value)| {
                let name_str = name.as_str().to_string();
                let value_str = value.to_str().ok()?.to_string();
                Some((name_str, value_str))
            })
            .collect();

        let cached = CachedResponse {
            status: parts.status.as_u16(),
            body: body_string.clone(),
            headers: headers_vec,
        };

        // Serialize and cache in Redis
        match serde_json::to_string(&cached) {
            Ok(json) => {
                let ttl = std::env::var("IDEMPOTENCY_TTL_SECONDS")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(DEFAULT_TTL_SECONDS);

                if let Err(e) = redis.set_ex::<_, _, ()>(&cache_key, json, ttl).await {
                    warn!(
                        message = "failed to cache response",
                        error = %e,
                        key = %idempotency_key
                    );
                }

                info!(
                    message = "idempotency response cached",
                    key = %idempotency_key,
                    status = parts.status.as_u16(),
                    ttl_seconds = ttl
                );
            }
            Err(e) => {
                warn!(
                    message = "failed to serialize response",
                    error = %e,
                    key = %idempotency_key
                );
            }
        }

        // Return response with reconstructed body
        Response::from_parts(parts, Body::from(body_bytes))
    } else {
        // Don't cache 5xx errors (server errors may be transient)
        response
    }
}
