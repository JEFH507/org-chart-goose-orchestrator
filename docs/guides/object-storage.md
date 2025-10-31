# Object Storage (Optional in Phase 0)
> See also: ADR 0014 (docs/adr/0014-ce-object-storage-default-and-provider-policy.md) — supersedes MinIO-for-CE detail in ADR‑0005.


Decision summary
- Object storage is OPT‑IN in Phase 0; OFF by default. The orchestrator is metadata‑only (ADRs 0005/0012), so storage isn’t required for the stack to be healthy.
- Support multiple S3‑compatible choices via compose profiles or env toggles to avoid lock‑in:
  - Apache‑licensed default option for CE: SeaweedFS (S3 gateway). Lightweight, ALv2.
  - Alternatives (AGPLv3): MinIO, Garage. Both widely used; include clear third‑party notices.
  - Heavier Apache option for scale: Apache Ozone (S3 gateway), not recommended for Phase 0 due to complexity.

License posture
- Using an AGPLv3 service (MinIO/Garage) as an external container does not affect your Apache‑2.0 licensed code, provided you don’t modify/redistribute AGPL binaries. Keep it optional and document licenses.

Configuration
- Use a single S3 config surface to swap providers:
  - S3_ENDPOINT, S3_ACCESS_KEY, S3_SECRET_KEY, S3_BUCKET, S3_REGION
- Ports (defaults; override via `.env.ce`):
  - MinIO: 9000 (S3), 9001 (console)
  - SeaweedFS: 8333 (S3), 9333 (master UI), 8081 (Filer API)

Recommendations
- Phase 0: leave OFF by default. Document SeaweedFS as the ALv2 default option. Include MinIO and Garage as alternatives with links and notices.
- Phase 1+: add a compose profile for each provider and a single env template.

References
- SeaweedFS: https://github.com/seaweedfs/seaweedfs (Apache‑2.0)
- MinIO: https://min.io/ (AGPLv3) — community discussion mixed on licensing
- Garage: https://garagehq.deuxfleurs.fr/ (AGPLv3)
- Apache Ozone: https://github.com/apache/ozone (Apache‑2.0)
- ADR‑0005/0012: metadata‑only server posture
