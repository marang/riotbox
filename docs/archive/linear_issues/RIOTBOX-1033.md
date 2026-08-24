# `RIOTBOX-1033` P012+: Strengthen Source Timing detector quality for arrangement use

- Ticket: `RIOTBOX-1033`
- Title: `P012+: Strengthen Source Timing detector quality for arrangement use`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1033/p012-strengthen-source-timing-detector-quality-for-arrangement-use`
- Project: `P012 | Source Timing Intelligence`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-30`
- Started: `2026-08-24`
- Finished: `2026-08-24`
- Branch: `feature/riotbox-1033-downbeat-phase-evidence`
- Linear branch: `feature/riotbox-1033-p012-strengthen-source-timing-detector-quality-for`
- Assignee: `Markus`
- Labels: `Audio`, `Improvement`, `benchmark`
- PR: `#1448 (https://github.com/marang/riotbox/pull/1448)`
- Merge commit: `595c8310dd66b290de2604c57e0eb168a5232f46`
- Deleted from Linear: `2026-08-24`
- Verification: `targeted Core timing tests; exact five-case registered Development matrix; live and restart Source Graph/observer proof; cargo test -p riotbox-core; strict Core clippy; just ci; GitHub rust-ci (PR #1448)`
- Docs touched: `docs/specs/source_timing_intelligence_spec.md, docs/reviews/riotbox_1033_repeated_loop_boundary_qualification_2026-08-24.md, docs/execution_roadmap.md, docs/phase_definition_of_done.md, docs/README.md, docs/research_decision_log.md (RBX-318)`
- Follow-ups: `Run a fresh corrected-current-state negative-family qualification before any weak_source or bad_timing promotion; preserve explicit manual confirmation and no automatic timing lock`

## Why This Ticket Existed

Beat20 exposed a musical downbeat ambiguity: raw accent evidence selected phase two even though repeated complete-bar structure and the file boundary supported phase zero, blocking honest downstream arrangement and negative-family handling.

## What Shipped

- Added the frozen source-timing-probe.repeated-loop-boundary-prior.v1 cue, which may reorder an already-ambiguous primary to phase zero without raising confidence, removing alternatives, or auto-confirming.
- Qualified the cue on the exact registered five-case Development matrix; only Beat20 activated it, while dense, sparse, tonal, and electronic contrasts retained their prior phases.
- Persisted typed cue provenance across primary and alternative hypotheses, retained explicit short-loop manual confirmation, and proved stopped transport, no generated fallback, and restart stability.

## Notes

- No Holdout, commercial reference, playback, automatic lock, demo, quality, release, hardness, or retroactive RIOTBOX-1459 promotion is claimed.
