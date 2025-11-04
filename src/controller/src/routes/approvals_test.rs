#[cfg(test)]
mod tests {
    use super::{submit_approval, SubmitApprovalRequest, SubmitApprovalResponse};
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
            .route("/approvals", axum::routing::post(submit_approval))
            .with_state(app_state)
    }

    #[tokio::test]
    async fn test_submit_approval_success() {
        let app = create_test_app();

        let payload = json!({
            "task_id": "task-001",
            "decision": "approved",
            "comments": "Looks good"
        });

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/approvals")
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(payload.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::ACCEPTED);
        
        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let result: SubmitApprovalResponse = serde_json::from_slice(&body).unwrap();
        assert!(result.approval_id.starts_with("approval-"));
    }

    #[tokio::test]
    async fn test_submit_approval_rejected() {
        let app = create_test_app();

        let payload = json!({
            "task_id": "task-001",
            "decision": "rejected",
            "comments": "Budget concerns"
        });

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/approvals")
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(payload.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::ACCEPTED);
    }

    #[tokio::test]
    async fn test_submit_approval_without_comment() {
        let app = create_test_app();

        let payload = json!({
            "task_id": "task-001",
            "decision": "approved"
        });

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/approvals")
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(payload.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::ACCEPTED);
    }

    #[tokio::test]
    async fn test_submit_approval_malformed_json() {
        let app = create_test_app();

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/approvals")
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from("{bad json"))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }
}
