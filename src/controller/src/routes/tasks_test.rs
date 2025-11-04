#[cfg(test)]
mod tests {
    use super::{route_task, RouteTaskRequest, RouteTaskResponse};
    use axum::{
        body::Body,
        http::{Request, StatusCode, header},
    };
    use serde_json::json;
    use tower::ServiceExt; // for `oneshot`
    use crate::{AppState, guard_client::GuardClient};
    use std::sync::Arc;

    fn create_test_app() -> axum::Router {
        let guard_client = Arc::new(GuardClient::from_env());
        let app_state = AppState::new(guard_client, None);
        axum::Router::new()
            .route("/tasks/route", axum::routing::post(route_task))
            .with_state(app_state)
    }

    #[tokio::test]
    async fn test_route_task_success() {
        let app = create_test_app();

        let payload = json!({
            "task": {
                "task_type": "approval",
                "description": "Budget approval request",
                "data": {"amount": 5000}
            },
            "target": "manager",
            "context": {"requestor": "alice@example.com"}
        });

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/tasks/route")
                    .header(header::CONTENT_TYPE, "application/json")
                    .header("Idempotency-Key", "550e8400-e29b-41d4-a716-446655440000")
                    .body(Body::from(payload.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::ACCEPTED);
    }

    #[tokio::test]
    async fn test_route_task_missing_idempotency_key() {
        let app = create_test_app();

        let payload = json!({
            "task": {"task_type": "approval", "data": {}},
            "target": "manager"
        });

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/tasks/route")
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(payload.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_route_task_invalid_idempotency_key() {
        let app = create_test_app();

        let payload = json!({
            "task": {"task_type": "approval", "data": {}},
            "target": "manager"
        });

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/tasks/route")
                    .header(header::CONTENT_TYPE, "application/json")
                    .header("Idempotency-Key", "not-a-uuid")
                    .body(Body::from(payload.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_route_task_with_trace_id() {
        let app = create_test_app();

        let payload = json!({
            "task": {"task_type": "approval", "data": {}},
            "target": "manager"
        });

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/tasks/route")
                    .header(header::CONTENT_TYPE, "application/json")
                    .header("Idempotency-Key", "550e8400-e29b-41d4-a716-446655440000")
                    .header("X-Trace-Id", "trace-12345")
                    .body(Body::from(payload.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::ACCEPTED);
    }

    #[tokio::test]
    async fn test_route_task_with_context() {
        let app = create_test_app();

        let payload = json!({
            "task": {
                "task_type": "approval",
                "description": "Purchase request",
                "data": {"amount": 5000}
            },
            "target": "manager",
            "context": {
                "requestor": "alice@example.com",
                "department": "sales",
                "priority": "high"
            }
        });

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/tasks/route")
                    .header(header::CONTENT_TYPE, "application/json")
                    .header("Idempotency-Key", "550e8400-e29b-41d4-a716-446655440000")
                    .body(Body::from(payload.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::ACCEPTED);
    }

    #[tokio::test]
    async fn test_route_task_malformed_json() {
        let app = create_test_app();

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/tasks/route")
                    .header(header::CONTENT_TYPE, "application/json")
                    .header("Idempotency-Key", "550e8400-e29b-41d4-a716-446655440000")
                    .body(Body::from("{invalid json"))
                    .unwrap(),
            )
            .await
            .unwrap();

        // Axum returns 422 for JSON deserialization errors
        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }
}
