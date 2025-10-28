# API

## Profile bundle (YAML)
- id, version, role, extensions_allowlist, recipes, prompts, env, policies, signatures:[{kid, sig}]

## Endpoints
- GET /api/v1/profiles/{role} → {bundle}
- POST /api/v1/policy/evaluate {subject, action, resource, context} → {allow:boolean, reason:string}

## Subject schema
- { tenantId, userId, roles:[...], labels:{dept, geo, clearance}, claims:{} }
