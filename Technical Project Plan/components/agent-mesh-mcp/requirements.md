# Requirements

## Functional
- send_task(target, task, context, policyHints) → taskId
- request_approval(sessionId, stepId, approver, payload) → approvalId
- notify(target, message, severity) → ack
- fetch_status(taskId|sessionId) → status

## Non-functional
- Security: Bearer JWT per call; include traceId; input size limits.
- Reliability: 3x retry with jitter; idempotency-key header.
