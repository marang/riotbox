# `RIOTBOX-77` Make W-30 internal resample taps audibly real on the current runtime seam

- Ticket: `RIOTBOX-77`
- Title: `Make W-30 internal resample taps audibly real on the current runtime seam`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-77/make-w-30-internal-resample-taps-audibly-real-on-the-current-runtime`
- Project: `P007 | W-30 MVP`
- Milestone: `W-30 MVP`
- Status: `Done`
- Created: `2026-04-17`
- Started: `2026-04-17`
- Finished: `2026-04-17`
- Branch: `feature/riotbox-77-w30-audible-resample-tap`
- Linear branch: `feature/riotbox-77-make-w-30-internal-resample-taps-audibly-real-on-the-current`
- Assignee: `Markus`
- Labels: `None`
- PR: `#71`
- Merge commit: `c8861c937d9c36a819772d7c7b881e4d76a6a450`
- Deleted from Linear: `Not deleted`
- Verification: `cargo fmt --all --check`, `cargo test`, `cargo clippy --all-targets --all-features -- -D warnings`, GitHub Actions `Rust CI` run `#194`
- Docs touched: `docs/research_decision_log.md`
- Follow-ups: `RIOTBOX-78`, `RIOTBOX-79`, `RIOTBOX-80`

## Why This Ticket Existed

The repo already had typed W-30 internal resample lineage state flowing through `riotbox-app`, but the audio callback still ignored it. The W-30 MVP needed one honest audible proof point on that existing seam before later resample-lab diagnostics and loop-freezer behavior could build on it safely.

## What Shipped

- threaded the shared `W30ResampleTapState` into the existing audio output callback path
- made the callback snapshot and mix one bounded internal-capture tap voice alongside the shipped TR-909 and W-30 preview renderers
- kept the new audible path on the current music-bus seam instead of opening a second W-30-only render loop
- added direct runtime tests for idle silence, audible lineage-ready taps, and zero-music-bus silence
- recorded the audio-seam decision in `docs/research_decision_log.md`

## Notes

- this slice intentionally stops at one bounded synthetic tap voice and does not yet add richer shell diagnostics or a dedicated resample audio fixture corpus
- later W-30 resample-lab work should keep extending this same callback seam rather than inventing a parallel renderer
