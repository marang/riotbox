# `RIOTBOX-1552` — Reconcile legacy two-bar recording proof and readiness rounding without rewriting V2 evidence

- Ticket: `RIOTBOX-1552`
- Title: `Reconcile legacy two-bar recording proof and readiness rounding without rewriting V2 evidence`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1552/reconcile-legacy-two-bar-recording-proof-and-readiness-rounding`
- Project: `P016 | Pro Workflow / Export`
- Milestone: `Not set in Linear`
- Status: `Done`
- Created: `2026-10-02`
- Started: `2026-10-02`
- Finished: `2026-10-02`
- Branch: `feature/riotbox-1552-canonical-two-bar-recording`
- Linear branch: `feature/riotbox-1552-reconcile-legacy-two-bar-recording-proof-and-readiness`
- Assignee: `Markus`
- Labels: `None`
- PR: [#1655](https://github.com/marang/riotbox/pull/1655)
- Merge commit: `175bc6ecc59e5a9dcab740384a4a405dc747596a`
- Deleted from Linear: `2026-10-16`
- Verification: Historical: final source-free `just ci`; Core recording 31/replay 122, App recording 21/DAW 15, CLI 152, UI 2 and 15 generated actual-clock combinations; native CI 37042597586 passed. Current archival check: Linear Done and merged PR metadata only.
- Docs touched: `docs/execution_roadmap.md`, `docs/research_decision_log.md`, `docs/reviews/riotbox_1552_canonical_two_bar_recording_2026-10-02.md`, `docs/specs/action_lexicon_spec.md`, `docs/specs/audio_core_spec.md`, `docs/specs/audio_qa/manifests_and_artifacts.md`, `docs/specs/session_file_spec.md`
- Follow-ups: At implementation closeout, RIOTBOX-1036 remained broader recording/export scope; RIOTBOX-1553 separately qualifies an eight-bar host take. Historical non-ready V2 receipts remain diagnosed, with no automatic migration or reassessment.

## Why This Ticket Existed

Compatibility review of RIOTBOX-1551 reproduced an inherited producer/consumer timing mismatch on main: physically valid two-bar takes could publish, commit/save/restore, yet fail full Core readiness and DAW handoff. At 48 kHz, 121.5 BPM serialized 42187 versus expected 42188 nanobeats; 166.5 BPM gave 57812/57813. This issue reconciles future timing authority without rewriting V2 evidence.

## What Shipped

- RBX-423 introduces two-bar recording V4 with explicit TwoBars, canonical runtime-f32/position-roundoff geometry and full readiness before commit. Future default/explicit-two-bar commands use V4; V3 remains eight/sixteen bars.
- Historical V1/V2/V3 identities and readiness are preserved, including existing non-ready V2 receipts. No legacy proof is relabeled, silently blessed, rerendered or automatically migrated.
- Existing V2 recording → DAWproject V1 remains; distinct V4 recording → DAWproject V2 reuses the same exact-byte archive writer, recorded source boundaries, transactional/no-clobber behavior and metadata-only restore.
- Queue pins the latest supported two-bar source receipt and required export version. Invalid latest evidence or version drift rejects without silently falling back to an older take.
- CLI/observer/existing UI labels project actual identities; no new CLI flag, TUI surface, realtime DSP or replayable external side effect was added.

## Notes

- Acceptance included fractional-f32 tempo/frame boundaries, actual callback arithmetic, physical start/half-frame/health gates, old identity readability, consumer readiness, rollback/no-clobber and synthetic clocks. Five tempo/rate cases × three callback partition patterns passed with exact frames/canonical serialization and no realtime code change.
- The first full CI passed behavior and synthetic gates before strict Clippy found a private generated-test enum naming issue. Only variants were renamed to LegacyV2/CanonicalV4/ExtendedV3, with no lint waiver or behavior change; the complete final CI rerun passed.
- Three independent review lanes (Spec/Evidence/API, adversarial restore/realtime and producer compatibility), excluding reviewers' own implementation, and final self-review retained zero actionable findings.
- Final reviewed head `3af6f30c6e76a554bde992e4a189b5001450f35b` passed native CI; merged and reviewed tree `479ae291d049413daae5f2254548e4c11ce8bac6` matched. Historical closeout reports main sync and exact branch cleanup, with the README from #1654 unchanged.
- Source-free tests only: no source/capture/Holdout/commercial reads or discovery, host playback, DAW import or fresh human/musical/source-general/release qualification. Prior host/listening evidence does not automatically transfer to V4. Existing Core/Session, Action and recorder/archive ownership is retained.

- Archive-only verification on 2026-10-16 re-read the full current Linear issue and all comments, checked attached GitHub PR bodies/branches/merge commits and successful retained native checks, and used tracked review metadata where cited. All implementation tests, benchmarks, source access, host and human outcomes above are historical evidence, not checks rerun by this cleanup. Linear's attachment reader does not fetch external GitHub URLs; those links were verified directly with `gh pr view`. Metadata dates use the Linear UTC timestamps.
