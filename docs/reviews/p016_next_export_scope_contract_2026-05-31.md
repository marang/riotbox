# P016 Next Export Scope Contract - 2026-05-31

Scope:

- `P016 | Pro Workflow / Export`
- RIOTBOX-1069
- next export scopes after the bounded `export.product_mix` action

Decision:

- Do not implement stem package, live recording, DAW session, or host-audio
  export from the current `full_grid_mix` receipt shape.
- The next implementation should first add a typed export artifact-set contract.
- Wider export scopes must extend Action Lexicon, Session/replay, observer, and
  audio-QA surfaces together.

## Current Boundary

Current implemented export:

- command: `export.product_mix`
- role: `full_grid_mix`
- boundary: deterministic Feral-grid generated-support proof
- artifact set: one product mix WAV plus proof path
- QA: product-export reproducibility smoke and hash validation
- observer: requested, started, completed, failed projection
- TUI: `E` queues the bounded action and Inspect reports receipt/failure

This is enough for a single reproducible product mix. It is not enough for
stems, live capture, DAW placement, or host-audio soak claims.

## Required Scope Additions

Stem package:

- separate command such as `export.stem_package`
- artifact roles, for example drums, bass, source support, generated support,
  and full mix
- per-artifact path/URI, media type, sample rate, channel count, duration
  frames, and hash
- source graph id/hash and capture lineage refs for source-backed or resampled
  stems
- QA gate proving each claimed stem role is non-silent, not fallback-collapsed,
  hash-stable, and role-labeled

Live recording:

- separate command such as `export.live_recording`
- explicit start/end transport state and recording duration
- real-session audio host/device evidence
- callback-gap, stream-error, and dropout summary
- WAV/hash proof for the captured recording
- no sandbox-only audio claim

DAW session:

- separate command such as `export.daw_session`
- arrangement scene refs, tempo map, bar/beat placement, and source-grid
  confidence
- track/artifact roles with stable hashes
- proof that placement follows Session/Source Graph timing truth
- no plugin-host or proprietary DAW claim unless a later contract names it

Host-audio soak:

- evidence gate for live recording/export readiness
- not a product export command by itself yet
- must record host API, device, duration, callback gaps, stream errors, and
  whether validation ran in a real user session

## Smallest Safe Next Slice

The next implementation should add the typed export artifact-set contract behind
the current product-mix path before adding new export commands. That gives
future stem package work a place to store per-artifact roles and hashes without
replacing export receipts or hiding state in `JamAppState`.

## Non-Goals

- no full stem package export in RIOTBOX-1069
- no DAW session writer
- no live host recording
- no automatic arranger export
- no Ghost export
- no second export truth outside ActionCommand, Session/replay, observer, and
  audio-QA contracts
