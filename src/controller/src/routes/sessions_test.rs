#[cfg(test)]
mod tests {
    use super::{list_sessions, create_session, CreateSessionRequest, CreateSessionResponse, SessionResponse};
    use axum::{
        body::Body,
        http::{Request, StatusCode, header},
    };
    use serde_json::json;
    use tower::ServiceExt;
    use crate::{AppState, guard_client::GuardClient};
    use std::sync::Arc;

    fn create_test_app() -> axum::Router {
        let guard_client = Arc::new(GuardClient::from_env());
        let app_state = AppState::new(guard_client, None);
        axum::Router::new()
            .route("/sessions", axum::routing::get(list_sessions))
            .route("/sessions", axum::routing::post(create_session))
            .with_state(app_state)
    }

    #[tokio::test]
    async fn test_list_sessions_empty() {
        let app = create_test_app();

        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/sessions")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        
        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let sessions: Vec<SessionResponse> = serde_json::from_slice(&body).unwrap();
        assert_eq!(sessions.len(), 0);
    }

    #[tokio::test]
    async fn test_create_session_success() {
        let app = create_test_app();

        let payload = json!({
            "agent_role": "finance"
        });

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/sessions")
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(payload.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::CREATED);
        
        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let result: CreateSessionResponse = serde_json::from_slice(&body).unwrap();
        assert!(result.session_id.starts_with("session-"));
    }

    #[tokio::test]
    async fn test_create_session_with_optional_metadata() {
        let app = create_test_app();

        let payload = json!({
            "agent_role": "finance",
            "metadata": {
                "requestor": "alice@example.com",
                "priority": "high"
            }
        });

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/sessions")
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(payload.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::CREATED);
    }

    #[tokio::test]
    async fn test_create_session_malformed_json() {
        let app = create_test_app();

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/sessions")
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from("{invalid"))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }
}
