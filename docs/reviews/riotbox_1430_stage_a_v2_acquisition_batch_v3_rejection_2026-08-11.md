# RIOTBOX-1430 Stage-A-v2 Acquisition Batch-v3 Rejection — 2026-08-11

## Verdict

`rejected_fail_closed_no_publication`

Batch v3 is consumed. OAuth authorization completed, but the first registered
original-file GET returned a non-200 response and stopped at the response-status
gate before any body byte was read. The temporary runner did not preserve the
actual status scalar, so it remains unknown and must not be reconstructed.

## Bounded Evidence

- attempt: `35987198-8410-4d08-92f6-3f605e6a210c`
- repository HEAD: `f5cb3843e911b1ed3080fcb7989a8095b7ead980`
- Batch-v3 raw SHA-256: `d56241539c057ddc8e4b62f719d9856360e5ab06067cf2db5860dc64bcae8067`
- ignored access-log raw SHA-256: `2e7068d86cf0391f6f316b92830ab88e0fb4420a35f705513625de6aaf1fa7f9`
- OAuth control-plane requests: `5`
- original-file request counts: `1/0`
- entry states: `requested`, `not_requested`, `not_requested`
- publication: `not_started`
- quarantine and final Batch-v3 directories: absent after rejection

No source body, source directory, audio decode, PCM iteration, source feature or
event computation, candidate/control render, playback, holdout audio, or
commercial reference was accessed. The failure is an authentication/HTTP gate
result only; it is not source, family, format, hardness, force, musical, or
human-listening evidence.

RBX-260 authorizes one complete Batch-v4 attempt with identical source
identities, API URLs, byte counts, destinations, and P2 header rules, changing
only to a source-free-verified in-memory web session. Batch v3 may not be
retried or reclassified.
