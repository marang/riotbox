# P011 Replay Hardening Checkpoint

Date: 2026-04-29
Scope: P011 Pro Hardening replay / restore work after RIOTBOX-426 through RIOTBOX-434.

## Verdict

P011 is active, not exit-ready.

The restore spine is now substantially safer than it was at P011 entry:

- accepted Ghost actions persist as normal actions plus structured commit records
- restore rebuilds `last_commit_boundary` from structured commit metadata
- restore rejects snapshot cursors beyond the action log
- restore rejects orphaned commit records
- restore rejects commit records for non-committed actions
- restore rejects missing or mismatched commit timestamps
- restore rejects duplicate commit records for the same action
- restore rejects invalid or duplicate commit sequence numbers inside a boundary

This is enough to treat commit-record metadata as a serious replay seam. It is not enough to claim full deterministic replay, crash recovery, or stage-run hardening.

## Evidence

- Replay model defines one replay truth and names commit records as structured durable commit metadata: `docs/specs/replay_model_spec.md`.
- Restore validation lives in the normal Jam app load path, not a side channel: `crates/riotbox-app/src/jam_app/persistence.rs`.
- Restore contract tests now live in a dedicated semantic test file: `crates/riotbox-app/src/jam_app/tests/restore_contracts.rs`.
- Replay-from-zero fixture proves commit boundary restore and queue cursor reservation survive a fresh load: `crates/riotbox-app/src/jam_app/tests/replay_hardening.rs`.
- Ghost accepted-action fixture proves accepted Ghost actions survive session roundtrip through the same action and commit-record path: `crates/riotbox-app/src/jam_app/tests/ghost_assist_queue.rs`.

## What Is Now Strong

- **Commit metadata is validated on load**: malformed commit records fail fast instead of silently becoming replay truth.
- **Boundary order is explicit**: replay consumers can use `commit_sequence` within a boundary instead of parsing result summaries or relying only on vector order.
- **Fresh restore preserves next-action safety**: the queue reserves action ids after persisted history, preventing accidental id reuse.
- **Test ownership improved**: restore contracts no longer live inside a mixed runtime-view test file.

## Remaining P011 Gaps

- **Full replay runner is not implemented**: Riotbox still hydrates from persisted runtime state plus validation rather than replaying every action from origin into a reconstructed state.
- **Snapshot convergence is not proven**: the repo does not yet compare latest-snapshot replay against full replay from origin.
- **Crash recovery is not exercised**: there is no interrupted-write or partial-session recovery fixture.
- **Export reproducibility is not proven as a P011 gate**: existing audio QA probes are useful, but not yet a full export determinism suite.
- **Long-run/stage-run hardening is incomplete**: current smoke probes are bounded and fast; they do not simulate extended live use.

## Recommended Next Slices

1. Add a minimal replay-plan builder that turns persisted action log plus commit records into deterministic committed-order entries without mutating runtime state.
2. Add a snapshot-vs-origin comparison fixture once replay-plan building is stable.
3. Add interrupted-session save fixtures for crash recovery.
4. Promote an existing first-playable probe into a longer stage-style run with explicit control/output assertions.
5. Add export reproducibility checks only after the relevant export surface is real enough to test.

## Checkpoint Decision

Continue P011, but switch from local restore validators to replay execution scaffolding next.

The next implementation should avoid a second replay architecture. It should reuse:

- `SessionFile`
- `ActionLog`
- `ActionCommitRecord`
- `CommitBoundaryState`
- normal queue / commit semantics
- existing audio QA evidence when a slice is audible
