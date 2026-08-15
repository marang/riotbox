# `RIOTBOX-1437` P023: Make TR-909 Slam audibly drive harder with source-backed counter-rhythm

- Ticket: `RIOTBOX-1437`
- Title: `P023: Make TR-909 Slam audibly drive harder with source-backed counter-rhythm`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1437/p023-make-tr-909-slam-audibly-drive-harder-with-source-backed-counter`
- Project: `P023 | Sound Excellence / Production Quality`
- Milestone: `M4 | Controlled Expansion`
- Status: `Done`
- Created: `2026-08-14`
- Started: `2026-08-14`
- Finished: `2026-08-15`
- Branch: `feature/riotbox-1437-p023-make-tr-909-slam-audibly-drive-harder-with-source`
- Linear branch: `feature/riotbox-1437-p023-make-tr-909-slam-audibly-drive-harder-with-source`
- Assignee: `Markus`
- Labels: `Audio`, `Feature`, `timing`
- PR: `#1394 (https://github.com/marang/riotbox/pull/1394)`
- Merge commit: `ae651c21c9459ffbf8d250ccc581a528d6b7d101`
- Deleted from Linear: `2026-08-15`
- Verification: `just ci: pass; GitHub rust-ci: pass; branch review: no open findings`
- Docs touched: `docs/benchmarks/tr909_counter_rhythm_slam_development_v1.json`, `docs/benchmarks/tr909_counter_rhythm_slam_development_v2.json`, `docs/benchmarks/tr909_counter_rhythm_slam_development_v3.json`, `docs/reviews/riotbox_1437_tr909_counter_rhythm_development_2026-08-14.md`, `docs/research_decision_log.md`
- Follow-ups: `A future audible slice must change rendered material, voice articulation, or source-backed arrangement/chop ownership; no v4 scalar retune.`

## Why This Ticket Existed

Test whether source-backed anchor/counter-rhythm topology could make the committed TR-909 Slam audibly drive harder after the collision-local RIOTBOX-1436 mechanism proved inaudible.

## What Shipped

- Immutable v1-v3 Development-only contracts and the exact terminal fail-closed qualification record.
- RBX-289 through RBX-292 preserve the modeling correction, final stronger topology, and prohibition on another scalar version.
- No counter-rhythm product behavior or unsupported musical-quality claim; existing Slam behavior remains unchanged.

## Notes

- Only dense passed v3; tonal correctly refused incompatible ownership and sparse missed the unchanged 0.02 exact-mix delta floor. No human playback, Holdout access, or commercial-reference access occurred.
