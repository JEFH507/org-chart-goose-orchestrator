use utoipa::OpenApi;
use utoipa::openapi::security::{SecurityScheme, HttpAuthScheme, HttpBuilder};

/// OpenAPI documentation for Controller API v1
#[derive(OpenApi)]
#[openapi(
    info(
        title = "Goose Controller API",
        version = "1.0.0",
        description = "Multi-agent orchestration controller API for goose-org-twin project",
        contact(
            name = "Goose Project",
            url = "https://github.com/JEFH507/org-chart-goose-orchestrator"
        )
    ),
    paths(
        crate::routes::tasks::route_task,
        crate::routes::sessions::list_sessions,
        crate::routes::sessions::create_session,
        crate::routes::approvals::submit_approval,
        crate::routes::profiles::get_profile,
        crate::routes::privacy::submit_audit_log,
        crate::status,
        crate::audit_ingest,
    ),
    components(
        schemas(
            crate::routes::tasks::RouteTaskRequest,
            crate::routes::tasks::RouteTaskResponse,
            crate::routes::tasks::TaskPayload,
            crate::routes::sessions::CreateSessionRequest,
            crate::routes::sessions::CreateSessionResponse,
            crate::routes::sessions::SessionResponse,
            crate::routes::approvals::SubmitApprovalRequest,
            crate::routes::approvals::SubmitApprovalResponse,
            crate::routes::privacy::AuditLogEntry,
            crate::routes::privacy::AuditLogResponse,
            // Phase 5: Profile endpoints now return Profile schema directly
            crate::StatusResponse,
            crate::AuditEvent,
        )
    ),
    tags(
        (name = "tasks", description = "Task routing and management"),
        (name = "sessions", description = "Session management"),
        (name = "approvals", description = "Approval workflows"),
        (name = "profiles", description = "Agent profile discovery"),
        (name = "privacy", description = "Privacy Guard audit logs"),
        (name = "system", description = "System health and audit"),
    ),
    modifiers(&SecurityAddon)
)]
pub struct ApiDoc;

/// Add JWT bearer authentication to OpenAPI spec
struct SecurityAddon;

impl utoipa::Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "bearer_auth",
                SecurityScheme::Http(
                    HttpBuilder::new()
                        .scheme(HttpAuthScheme::Bearer)
                        .bearer_format("JWT")
                        .build()
                )
            )
        }
    }
}
