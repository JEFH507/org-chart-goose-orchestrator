# ADR 0014: CE Object Storage Default and Provider Policy

- Status: Accepted (MVP)
- Date: 2025-10-31
- Supersedes: Detail in ADR-0005 regarding “MinIO for CE”
- Authors: @owner

## Context
ADR-0005/0012 establish a metadata-only server posture and mention S3-compatible storage (with MinIO as an example for CE). To reduce licensing friction (AGPLv3) and preserve an Apache-2.0 friendly baseline while keeping choice and avoiding lock-in, we need a precise CE policy for object storage providers.

## Decision
- Phase 0: Object storage is OFF by default (opt-in). The stack must be healthy without S3.
- CE default option: SeaweedFS (Apache-2.0) S3 gateway.
- Alternatives (optional): MinIO (AGPLv3), Garage (AGPLv3). Both are widely used and supported; document license notes and keep optional.
- Heavier Apache option for scale: Apache Ozone S3 gateway; not recommended for Phase 0 due to complexity.
- Provide a single S3 configuration surface (endpoint, access key, secret key, bucket, region) so providers are easily swappable without code changes.

## Consequences
- Compose profiles for SeaweedFS/MinIO/Garage will be added starting in Phase 1; Phase 0 documents choice and keeps storage off by default.
- Guides and examples refer to “S3-compatible” generally; SeaweedFS is the documented CE default option.
- Enterprise teams with AGPL policies can select SeaweedFS or Ozone to keep a clean ALv2 story.

## Alignment
- ADR-0005/0012: Metadata-only server; storage optional.
- Licensing posture: preserves Apache-2.0 friendliness and minimizes procurement friction while keeping choice.

## References
- Guides: docs/guides/object-storage.md
- Third-party notices: docs/THIRD_PARTY.md
- SeaweedFS: https://github.com/seaweedfs/seaweedfs
- MinIO: https://min.io/
- Garage: https://garagehq.deuxfleurs.fr/
- Apache Ozone: https://github.com/apache/ozone
