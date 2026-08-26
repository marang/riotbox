# RIOTBOX-1476 Product-Mix Export Proof Handoff Review — 2026-08-26

## Scope

- Phase: P016 / Pro Workflow / Export
- Issue: RIOTBOX-1476
- Boundary: existing deterministic Feral-grid `full_grid_mix`
- Explicit non-goals: stems, DAW session export, live recording, host-audio
  capture, new rendering behavior, and new audible-content claims

## Problem Confirmed

The Jam export control queued the immediate `export.product_mix` side-effect
action, but the live event loop never invoked the existing proof-backed commit
transaction. A request could therefore remain pending indefinitely. The lower
transaction also accepted a proof without comparing its source hash to the
active Source Graph before attaching current Session lineage.

## Shipped

- Interactive launches accept a paired proof/destination handoff.
- The Jam export control executes the existing side effect outside the realtime
  callback and reports receipt or rejection feedback through existing state.
- Product export validates proof source identity before destination creation;
  `sha256:`-prefixed and raw hash identities normalize to the same value.
- Missing handoff, missing active Source Graph, source mismatch, invalid proof
  or artifact, and destination failure reject without a receipt or pending
  export action.
- Existing destination files are never overwritten. An already complete,
  hash-identical bundle is idempotent; incomplete or different bundles reject
  before mutation.
- Active Source Graph and timing-grid lineage attach only after source identity
  validation. Developer proof transactions without an active graph no longer
  borrow Session lineage.
- `just product-export-handoff <source> <destination>` creates a reusable,
  contained proof bundle by rendering the same explicit source twice and
  atomically publishing only after reproducibility validation. Existing
  destinations are refused before rendering.
- The product-mix commit transaction now owns a dedicated semantic Rust module;
  the mixed export module is 303 lines and the commit module is 276 lines.

## Verification

- `cargo test -p riotbox-app`: pass; primary library suite 655/655, plus all
  binary and integration suites
- focused product-export tests: pass
- synthetic `just product-export-handoff`: pass; declared and actual
  `full_grid_mix` SHA-256 identities match
- existing handoff destination mutation: rejected before rerender
- `just ci`: pass, source-free
- `cargo fmt --all -- --check`: pass
- `git diff --check`: pass

## Review Result

No correctness, product-spine, realtime-boundary, replay, observer, or module
ownership blocker remains in the reviewed diff. Exported audio bytes are copied
hash-identically from the already validated artifact; this slice changes no
sound and requires no human listening review.
