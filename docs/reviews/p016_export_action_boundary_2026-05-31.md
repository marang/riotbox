# P016 Export Action Boundary - 2026-05-31

Scope:

- `P016 | Pro Workflow / Export`
- RIOTBOX-1061 through RIOTBOX-1063
- future user-triggered export action boundary
- current `riotbox-core::export_readiness` contract

Decision:

- The first P016 export action should target the current `full_grid_mix`
  product-export role.
- The proof boundary remains the deterministic Feral-grid generated-support
  export validated by `just product-export-reproducibility-smoke`.
- The export should write a product mix plus proof/manifest receipt, not a stem
  package, live recording, DAW session, or host-audio capture.
- This review does not add a runtime action or file-writing workflow.

## Future Action Shape

Reserved command name:

- `export.product_mix`

Intended command boundary:

- actor: `user` for explicit export requests, `system` only for deterministic QA
  drills
- target scope: `session`
- quantization: `immediate`
- params:
  - `export_role`: initially `full_grid_mix`
  - `boundary`: initially `feral-grid generated-support export`
  - `include_manifest`: initially `true`
  - `destination_kind`: initially local file path / artifact directory
- result:
  - export receipt id
  - exported artifact path
  - manifest/proof path
  - export hash
  - normalized manifest hash
  - unsupported-scope flags
- undo policy: not undoable; deleting a written export is a filesystem concern,
  not a musical undo

The action must not run blocking file I/O, rendering, hashing, manifest writing,
or proof validation on the realtime audio callback.

## Session And Replay Boundary

The future action should create an export receipt in session/replay state once
the side effect succeeds.

Minimum receipt fields:

- receipt id
- created by action id
- created at timestamp
- export role
- export boundary
- artifact path or artifact URI
- export hash
- normalized manifest hash
- readiness status
- unsupported-scope flags copied from `ExportReadinessContract`

Replay must not blindly rewrite files as a hidden side effect. A replay path may
validate receipt metadata, rebuild an export on explicit request, or report a
missing artifact, but the action log remains the product truth for what the user
asked Riotbox to export.

## Observer And QA Boundary

Observer events should distinguish:

- export requested
- export started
- export completed
- export failed

The output path must be proven with:

- `just product-export-reproducibility-smoke`
- manifest/proof validation
- non-collapsed output metrics inherited from the Feral-grid manifest
- receipt serialization / replay fixture coverage once receipts exist

## Explicit Non-Goals

The first export action must not claim:

- stem package export
- live recording export
- DAW session export
- host-audio soak
- automatic arranger export
- automatic Ghost export
- arbitrary source polish

Those remain later P016/P017/P018 work and must extend the same Action Lexicon,
Session/replay, observer, and audio-QA seams.

## Follow-Up Tickets

- Implement the first `export.product_mix` action and export receipt model.
- Surface export receipts in Jam/System inspect without adding a second export
  truth.
- Add receipt replay/restore fixture coverage before widening export scope.
