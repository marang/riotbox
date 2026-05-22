# `RIOTBOX-936` Pin Beat20 real-source downbeat ambiguity proof fields

- Ticket: `RIOTBOX-936`
- Title: `Pin Beat20 real-source downbeat ambiguity proof fields`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-936/pin-beat20-real-source-downbeat-ambiguity-proof-fields`
- Project: `P012 | Source Timing Intelligence`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-22`
- Started: `2026-05-22`
- Finished: `2026-05-22`
- Branch: `feature/riotbox-936-beat20-downbeat-ambiguity-proof`
- Linear branch: `feature/riotbox-936-pin-beat20-real-source-downbeat-ambiguity-proof-fields`
- Assignee: `Markus`
- Labels: `timing`
- PR: `#929 (https://github.com/marang/riotbox/pull/929)`
- Merge commit: `eb63ae1c32adb6e4f3796c5ce4c3bd348a5e64bf`
- Deleted from Linear: `2026-05-22`
- Verification: `just beat20-auto-feral-grid-fallback-proof local-beat20-feral-grid-auto-fallback-proof; just recipe15-feral-grid-auto-proof; git diff --check; GitHub Actions Rust CI run 26293665171 passed`
- Docs touched: `docs/benchmarks/beat20_auto_feral_grid_fallback_2026-05-21.md`
- Follow-ups: `Continue P012 source-timing spine with the next smallest roadmap-aligned proof after current PR closeout.`

## Why This Ticket Existed

Pin Beat20 real-source downbeat ambiguity proof fields so reviewers can see why ambiguous bar phase remains manual-confirm/static-default.

## What Shipped

- Tightened the Beat20 auto Feral-grid fallback proof to assert primary downbeat score, primary downbeat margin, and alternate downbeat phase count; updated the Beat20 benchmark note with captured values.

## Notes

- No analyzer, ActionCommand, Session, JamAppState, realtime audio, or generated audio behavior changed; this only tightens existing QA proof expectations.
