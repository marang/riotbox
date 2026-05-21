# `RIOTBOX-868` Add aggregate Recipe 15 Feral grid auto proof target

- Ticket: `RIOTBOX-868`
- Title: `Add aggregate Recipe 15 Feral grid auto proof target`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-868/add-aggregate-recipe-15-feral-grid-auto-proof-target`
- Project: `P012 | Source Timing Intelligence`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-21`
- Started: `2026-05-21`
- Finished: `2026-05-21`
- Branch: `feature/riotbox-868-add-aggregate-recipe-15-feral-grid-auto-proof-target`
- Linear branch: `feature/riotbox-868-add-aggregate-recipe-15-feral-grid-auto-proof-target`
- Assignee: `Markus`
- Labels: `benchmark`, `ux`
- PR: `#862 (https://github.com/marang/riotbox/pull/862)`
- Merge commit: `d16a918fa35bfa2311e22f2d34f74bbfb1d2af66`
- Deleted from Linear: `2026-05-21`
- Verification: `git diff --check; scripts/run_compact.sh /tmp/riotbox868-recipe15-proof.log just recipe15-feral-grid-auto-proof; GitHub Actions Rust CI success on PR #862`
- Docs touched: `Justfile`, `docs/jam_recipes.md`
- Follow-ups: `None`

## Why This Ticket Existed

Recipe 15 had four separate local proof targets after RIOTBOX-864 through RIOTBOX-867; users and reviewers needed one aggregate command for the full current auto-BPM contract.

## What Shipped

- Added just recipe15-feral-grid-auto-proof and linked it from Recipe 15 so the full Beat03, Beat08, DH_BeatC, and Beat20 fallback proof set runs in one command.

## Notes

- No analyzer, ActionCommand, Session, JamAppState, or realtime audio behavior changed. Aggregate proof covers three source_timing cautious short-loop paths and one static_default ambiguous-downbeat fallback.
