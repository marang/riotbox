# P011 Replay Hardening Checkpoint

Date: 2026-04-29
Scope: P011 Pro Hardening replay / restore / QA-gate work after RIOTBOX-426 through RIOTBOX-461.

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

After RIOTBOX-445 through RIOTBOX-461, P011 also has bounded evidence for stage-style audio QA, latest-snapshot replay planning, partial-session failure behavior, offline-render reproducibility, and graph-aware Scene movement replay. These are real hardening gains, but most remain command-family or smoke-level gates. They should narrow the next work, not be mistaken for full replay execution, full crash recovery, or full export.

## Evidence

- Replay model defines one replay truth and names commit records as structured durable commit metadata: `docs/specs/replay_model_spec.md`.
- Restore validation lives in the normal Jam app load path, not a side channel: `crates/riotbox-app/src/jam_app/persistence.rs`.
- Restore contract tests now live in a dedicated semantic test file: `crates/riotbox-app/src/jam_app/tests/restore_contracts.rs`.
- Replay-from-zero fixture proves commit boundary restore and queue cursor reservation survive a fresh load: `crates/riotbox-app/src/jam_app/tests/replay_hardening.rs`.
- Ghost accepted-action fixture proves accepted Ghost actions survive session roundtrip through the same action and commit-record path: `crates/riotbox-app/src/jam_app/tests/ghost_assist_queue.rs`.
- Target replay dry-run and latest-snapshot convergence summaries expose replay scope without mutating runtime state: `crates/riotbox-core/src/replay/summary.rs`.
- Accepted-Ghost app replay fixture now exercises latest-snapshot convergence summaries on restored session data: `crates/riotbox-app/src/jam_app/tests/replay_hardening.rs`.
- Stage-style Jam probe adds a longer generated W-30 source-diff output plus observer/audio correlation gate: `scripts/validate_stage_style_jam_probe.sh`.
- Partial-session load regression proves truncated JSON fails explicitly and leaves adjacent valid session data loadable: `crates/riotbox-core/src/persistence.rs`.
- Offline-render reproducibility smoke proves a deterministic source-backed W-30 render emits byte-stable WAV output: `scripts/validate_offline_render_reproducibility.sh`.
- MC-202, TR-909, the first W-30 cue, and the first Scene launch/restore replay slices now include app-level render or preview checks, proving replayed state feeds downstream audible projection for those bounded action families.

## What Is Now Strong

- **Commit metadata is validated on load**: malformed commit records fail fast instead of silently becoming replay truth.
- **Boundary order is explicit**: replay consumers can use `commit_sequence` within a boundary instead of parsing result summaries or relying only on vector order.
- **Fresh restore preserves next-action safety**: the queue reserves action ids after persisted history, preventing accidental id reuse.
- **Test ownership improved**: restore contracts no longer live inside a mixed runtime-view test file.
- **Replay planning is inspectable**: dry-run and convergence summaries can show origin, anchor, suffix, and full-replay requirements without becoming a second replay engine.
- **Smoke-level audio QA is broader**: first-playable, stage-style, observer/audio correlation, and offline render reproducibility gates now run under `just audio-qa-ci`.
- **Crash failure behavior is explicit at MVP level**: partial JSON is rejected instead of silently repaired or misloaded.

## Remaining P011 Gaps

- **Full replay runner is not implemented**: Riotbox still hydrates from persisted runtime state plus validation rather than replaying every action from origin into a reconstructed state.
- **Snapshot convergence is partially execution-proven**: current helpers prove selected anchor and suffix scope. The safe structural replay subset, MC-202 phrase-family subset, first TR-909 support-move subset, first W-30 cue subset, and first Scene launch/restore subset now compare origin replay against materialized anchor-plus-suffix replay. Graph-aware Scene movement hydration now exists for the launch/restore subset. Real snapshot payload hydration and broader W-30 / scene replay are still open.
- **Crash recovery is bounded, not complete**: saves are serialize-then-temp-rename and truncated JSON fails safely. MVP policy rejects hidden automatic repair, and a non-mutating scanner plus app-facing manual recovery surface can now report canonical, orphan-temp, and autosave candidates with validation status and trust hints. There is still no interactive guided TUI fallback prompt, automatic fallback selection, journal, or interrupted multi-file recovery.
- **Export reproducibility is only a smoke gate**: current evidence proves one deterministic offline W-30 render is byte-stable, not full arrangement export, stems, recording, or manifest normalization.
- **Long-run/stage-run hardening is still bounded**: `just stage-style-jam-probe` is a longer CI probe, not a soak test or extended live-session simulation.

## Recommended Next Slices

1. Continue expanding replay-from-origin vs latest-snapshot convergence one replay-safe musical action family at a time, with output-path proof where audible state is affected. MC-202 phrase replay, the first TR-909 support-move family, the first W-30 cue subset, and graph-aware Scene launch/restore movement replay are now covered; W-30 artifact-producing behavior and broader Scene behavior remain open.
2. Add an interactive guided TUI/manual recovery prompt on top of the existing recovery surface without adding automatic fallback selection.
3. Keep replay executor expansion command-bounded; do not jump from MC-202 phrase replay to full lexicon replay without family-level convergence and output proof.
4. Expand stage-style QA only where it catches new risk: multi-boundary observer fixtures, longer action sequences, or real user-session observer correlation.
5. Grow export reproducibility only when a product export surface exists; until then, keep offline render hash checks scoped and clearly labeled.

## Checkpoint Decision

Continue P011, but switch from validators and smoke gates to replay execution scaffolding next.

The next implementation should avoid a second replay architecture. It should reuse:

- `SessionFile`
- `ActionLog`
- `ActionCommitRecord`
- `CommitBoundaryState`
- normal queue / commit semantics
- existing audio QA evidence when a slice is audible
