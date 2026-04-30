# Snapshot Payload Hydration Boundary

Date: 2026-04-30  
Ticket: RIOTBOX-476  
Scope: P011 Pro Hardening planning boundary for real snapshot payload hydration.

## Verdict

Riotbox is ready for a narrow snapshot-payload implementation, but not for a
broad restore runner.

The safe boundary is:

- store a typed, versioned snapshot payload that contains replay-relevant
  `RuntimeState` at the snapshot cursor
- validate that payload against the selected snapshot metadata and action log
- hydrate a cloned session to the anchor runtime state
- reuse the existing target-suffix executor for all actions after the anchor
- rebuild app/TUI derived state only after the suffix succeeds

This keeps snapshots as restore accelerators and preserves the existing action
log / commit-record replay spine.

## Current Seam

- `SessionFile.runtime_state` already stores the latest materialized musical
  state.
- `SessionFile.snapshots` currently store only `snapshot_id`, `created_at`,
  `label`, and `action_cursor`.
- Replay planning can select the latest snapshot at or before a target cursor.
- Target replay suffix execution exists, but it assumes the caller already
  hydrated the session to the anchor state.
- Tests currently materialize anchors by replaying prefixes; that proves executor
  behavior, not persisted snapshot payload hydration.

## Minimal Payload Shape

The next code slice should add a versioned payload to `Snapshot`.

Initial fields:

- `payload_version`
- `snapshot_id`
- `action_cursor`
- `runtime_state`

The payload should reuse `RuntimeState` instead of introducing a parallel lane,
scene, mixer, transport, or undo model. If future snapshots need artifact
manifests, source graph stamps, or compact deltas, those should be additive
payload fields behind versioned validation.

## Hydration Rules

Hydration must be explicit and all-or-nothing:

- do not mutate the live session until payload validation and suffix execution
  both succeed
- reject unsupported payload versions
- reject cursor and snapshot-id mismatches
- reject unresolved source, graph, capture, or pad references that the payload
  requires
- do not recreate capture artifacts during hydration
- do not rerun analysis or Ghost
- do not silently pick a different anchor when payload hydration fails

The restore surface may expose a read-only prompt explaining the failed anchor
and available recovery candidates. Automatic recovery selection remains out of
scope.

## Next Implementation Ticket

Linear: RIOTBOX-477

Create a small P011 implementation ticket for:

- adding the typed snapshot payload scaffold
- validating payload identity and cursor alignment
- hydrating a cloned session from the selected snapshot payload
- applying the existing suffix executor afterward
- proving convergence against the latest runtime state for one replay-safe
  fixture

Out of scope for that ticket:

- full replay runner
- W-30 capture/resample artifact recreation
- automatic recovery selection
- export or offline re-render work
