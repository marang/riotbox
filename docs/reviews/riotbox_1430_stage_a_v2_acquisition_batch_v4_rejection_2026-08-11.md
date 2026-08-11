# RIOTBOX-1430 Stage-A-v2 Acquisition Batch-v4 Rejection — 2026-08-11

## Verdict

`rejected_fail_closed_no_publication`

Batch v4 is consumed. Its source-free web-session proof passed, but the first
registered original-file GET returned HTTP `406` before any body byte was read.
The runner had sent `Accept: application/octet-stream`; Freesound's REST content
negotiation does not register that request media type.

## Bounded Evidence

- attempt: `e2aaf002-466a-4a9f-a7be-400226743528`
- repository HEAD: `7a112d62b11bb7791d17269e1902970a8a22ef96`
- Batch-v4 raw SHA-256: `24a5c6aa82f08ed4318f6167cbf2abb0e9b3ac39a49bca1513bf7fe3d0aa42ad`
- ignored access-log raw SHA-256: `588ad3a3bc775536d37682e471b85f5415595ee1e3ee62864e7166b6a269c30c`
- session control-plane requests: `3`
- original-file request counts: `1/0`
- response: status `406`, JSON, `57` declared bytes, no redirect
- entry states: `response_received`, `not_requested`, `not_requested`
- publication: `not_started`
- quarantine and final Batch-v4 directories: absent after rejection

The response body remained unread. No source directory, audio decode, PCM
iteration, source feature/event computation, render, playback, holdout audio,
or commercial reference was accessed. RBX-261 authorizes Batch v5 with the
same identities and gates, changing only the request Accept value to `*/*`.
