# Risks

- PII leakage → validation layer rejects events with raw PII. Hash/regex checks.
- Volume spikes → backpressure and batch ingest.
