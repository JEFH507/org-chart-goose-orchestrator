# API

## Tool input JSON schemas (examples)
- send_task:
  - { target: {role?:string, agentId?:string}, task:{type:string, payload:any}, context?:{sessionId?:string, breadcrumbs?:[...]}, policyHints?:{classification?:string, sensitivity?:string}, idempotencyKey?:string }
- fetch_status:
  - { taskId?:string, sessionId?:string }

## Responses
- { id:string, status:"accepted"|"rejected", reason?:string }

## Events (audit)
- RouteEvent, ApprovalEvent
