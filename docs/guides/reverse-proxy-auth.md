# Reverse Proxy Auth Pattern (Phase 1.2)

## Overview

Per ADR-0019, the MVP does **not include a dedicated auth gateway container**. Instead, the controller performs JWT verification directly.

If you deploy a reverse proxy (nginx, Traefik, Caddy, etc.) in front of the controller, it should:
1. Pass through the `Authorization` header unchanged
2. Optionally enforce TLS termination
3. Optionally add request logging or rate limiting

The controller will validate the JWT on protected endpoints (`/audit/ingest`).

## Pattern: Pass-Through Authorization

```nginx
# Example nginx config (simplified)
upstream controller {
    server controller:8088;
}

server {
    listen 443 ssl;
    server_name controller.example.com;

    # TLS config here (ssl_certificate, ssl_certificate_key)

    location / {
        proxy_pass http://controller;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        
        # IMPORTANT: Pass Authorization header through
        proxy_pass_request_headers on;
    }
}
```

## Traefik Example

```yaml
# docker-compose.yml snippet
services:
  traefik:
    image: traefik:v3.0
    command:
      - --providers.docker=true
      - --entrypoints.web.address=:80
      - --entrypoints.websecure.address=:443
    ports:
      - "80:80"
      - "443:443"
    volumes:
      - /var/run/docker.sock:/var/run/docker.sock:ro

  controller:
    labels:
      - "traefik.enable=true"
      - "traefik.http.routers.controller.rule=Host(`controller.example.com`)"
      - "traefik.http.routers.controller.entrypoints=websecure"
      - "traefik.http.routers.controller.tls=true"
      # No auth middleware on Traefik side; controller handles JWT
```

## What the Reverse Proxy Should NOT Do

- ❌ Strip or modify the `Authorization` header
- ❌ Perform JWT validation (controller does this)
- ❌ Add its own authentication layer (unless you have org-specific requirements)

The controller is designed to validate JWTs independently per ADR-0019.

## Optional: Reverse Proxy Auth (Advanced)

If your organization requires edge validation:
1. Configure the reverse proxy to validate JWT and add claims as headers (e.g., `X-Auth-Subject`)
2. The controller can optionally trust these headers if configured to do so
3. This is **out of scope for Phase 1.2** and would require custom middleware

For MVP, stick with the pass-through pattern above.

## Development (No Proxy)

In local development, access the controller directly:
```bash
curl http://localhost:8088/status
curl -H "Authorization: Bearer <jwt>" -X POST http://localhost:8088/audit/ingest -d '...'
```

## References
- ADR-0019 (Auth Bridge and JWT Verification)
- docs/tests/smoke-phase1.2.md (end-to-end JWT testing)
