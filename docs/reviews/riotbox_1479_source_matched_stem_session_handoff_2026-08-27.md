# RIOTBOX-1479 Source-Matched Stem Session Handoff Review — 2026-08-27

## Scope

- Phase: P016 / Pro Workflow / Export
- Issue: RIOTBOX-1479
- Boundary: non-interactive operator ingress from an exact Development V2
  product-stem handoff into the existing Action/Session/observer spine
- Explicit non-goals: new rendering or DSP, role or threshold changes, TUI or
  Ghost musician controls, DAW placement, live recording, source-general
  quality, Holdout access, release readiness, and human listening

## Implemented

- Core now parses and validates the exact `riotbox.product_stem_handoff.v2`
  schema with strict typed roles, identities, paths, grid, reconstruction, and
  MC-202 source-expression evidence.
- `export.stem_package` has a distinct `source_matched_handoff_v1` action
  boundary and explicit proof path. Its fixed role set is drums, music, and
  bass; role overrides are rejected.
- The operator ingress requires the proof source SHA-256 to match the active
  Source Graph and requires a Session graph ref with the same source id, graph
  version, and canonical graph hash before any destination write. A matching
  confirmed timing grid is attached when present.
- Every declared artifact is a contained regular file with its expected hash,
  PCM16 format, and exact grid. The three-stem sum is recalculated against the
  unchanged full mix and the frozen declared metrics.
- The writer copies the declared stem bytes through an isolated staging tree,
  measures the written files, builds the existing manifest/proof and QA gates,
  and promotes only a complete ready package. It does not overwrite an existing
  package.
- Each stem carries source lineage, the V2 normalized-manifest identity, and a
  source-vs-fallback comparison against missing-source fail-closed silence. The
  comparison reference binds the exact V2 proof and role; measured stem RMS is
  the signal difference, with no meaningless silence correlation claim.
- Success commits one Action, commit record, ready Session receipt, and observer
  lifecycle. Failure rejects the pending action and leaves no final package or
  receipt. Replay never rewrites files.
- The shared musician surface recognizes the new operator receipt identity but
  remains disabled behind operator-proof-only, DAW-placement, and structured-
  listening blockers.

## Evidence

- Core library tests: 437/437 pass
- App library tests: 665/665 pass
- Focused source-matched app/CLI tests: 10/10 pass
- Existing Python V2 contract mutations: 13/13 pass
- Complete source-free `just ci`: pass
- Positive proof covers three written WAVs byte-identical to their V2 inputs,
  all existing readiness gates, committed Action/Session truth, timing/source
  lineage, and the still-disabled musician surface.
- Negative proofs cover stale active source, missing Session graph lineage,
  symlinked input, and a competing pending stem action before destination
  creation, plus artifact-hash and reconstruction drift with no final package
  or receipt.
- No registered Development source, source directory, Holdout, commercial
  reference, audio playback, renderer change, or threshold tuning was used.

## Review Boundary

This slice completes the source-matched Foundation handoff into committed
Session truth. It does not say that V2 is release material or make stem export a
musician-facing control. DAW placement and structured listening remain explicit
next gates, and the Development producer's readiness flags remain false.
