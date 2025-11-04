#[cfg(test)]
mod tests {
    use super::{get_profile, ProfileResponse};
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use tower::ServiceExt;
    use crate::{AppState, guard_client::GuardClient};
    use std::sync::Arc;

    fn create_test_app() -> axum::Router {
        let guard_client = Arc::new(GuardClient::from_env());
        let app_state = AppState::new(guard_client, None);
        axum::Router::new()
            .route("/profiles/:role", axum::routing::get(get_profile))
            .with_state(app_state)
    }

    #[tokio::test]
    async fn test_get_profile_manager() {
        let app = create_test_app();

        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/profiles/manager")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        
        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let profile: ProfileResponse = serde_json::from_slice(&body).unwrap();
        assert_eq!(profile.role, "manager");
        assert!(profile.capabilities.contains(&"task_routing".to_string()));
        assert!(profile.capabilities.contains(&"approval_workflow".to_string()));
    }

    #[tokio::test]
    async fn test_get_profile_finance() {
        let app = create_test_app();

        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/profiles/finance")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        
        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let profile: ProfileResponse = serde_json::from_slice(&body).unwrap();
        assert_eq!(profile.role, "finance");
        assert!(profile.capabilities.contains(&"budget_requests".to_string()));
        assert!(profile.capabilities.contains(&"expense_tracking".to_string()));
    }

    #[tokio::test]
    async fn test_get_profile_engineering() {
        let app = create_test_app();

        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/profiles/engineering")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        
        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let profile: ProfileResponse = serde_json::from_slice(&body).unwrap();
        assert_eq!(profile.role, "engineering");
        assert!(profile.capabilities.contains(&"code_review".to_string()));
        assert!(profile.capabilities.contains(&"deployment".to_string()));
    }

    #[tokio::test]
    async fn test_get_profile_unknown_role() {
        let app = create_test_app();

        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/profiles/unknown-role")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        
        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let profile: ProfileResponse = serde_json::from_slice(&body).unwrap();
        assert_eq!(profile.role, "unknown-role");
        assert!(profile.capabilities.contains(&"task_routing".to_string()));
    }
}
