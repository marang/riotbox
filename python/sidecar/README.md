# Riotbox Python Sidecar

This directory contains the bounded Python analysis sidecar used by the Rust
ingest seam.

Current contents:

- `json_stdio_sidecar.py`
  Versioned newline-delimited JSON over `stdio`, a transport-only stub provider,
  and the current decoded-PCM-WAV baseline provider
- `test_json_stdio_sidecar.py`
  Deterministic clock/provenance contract tests

The process is a measurement/provider boundary, not product intelligence. Rust
validates protocol compatibility before trusting a graph. Production graphs are
stamped once with current UTC generation time; tests inject a fixed clock. The
`stub.transport` provider and `stub_transport_only` warning are scaffolding and
must never be presented as source-derived, release/demo-ready, or quality proof.
