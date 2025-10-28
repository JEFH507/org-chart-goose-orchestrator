# API

## Endpoints
- POST /api/v1/tasks/route
  - req: {task, target?, context?, policyHints?}
  - res: {taskId, routedTo:[{agentId, role}]}
- POST /api/v1/approvals
  - req: {sessionId, stepId, approverRole, payload, decision?}
  - res: {approvalId, status}
- GET /api/v1/status/{id} → {status, progress, lastUpdate}

OpenAPI: schemas for Task, Session, Approval, AuditEvent.
