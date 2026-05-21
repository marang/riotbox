# `RIOTBOX-870` Fold Recipe 15 auto timing proof into the P012 all-lane output gate

- Ticket: `RIOTBOX-870`
- Title: `Fold Recipe 15 auto timing proof into the P012 all-lane output gate`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-870/fold-recipe-15-auto-timing-proof-into-the-p012-all-lane-output-gate`
- Project: `P012 | Source Timing Intelligence`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-21`
- Started: `2026-05-21`
- Finished: `2026-05-21`
- Branch: `feature/riotbox-870-fold-recipe-15-auto-timing-proof-into-p012-all-lane-output-gate`
- Linear branch: `feature/riotbox-870-fold-recipe-15-auto-timing-proof-into-the-p012-all-lane`
- Assignee: `Markus`
- Labels: `Audio`, `benchmark`, `timing`
- PR: `#864 (https://github.com/marang/riotbox/pull/864)`
- Merge commit: `386aa098d6d7951cc9dad0278f2cb1ac7f2e3d5f`
- Deleted from Linear: `2026-05-21`
- Verification: `git diff --check`; `just p012-all-lane-source-grid-output-proof`; `just ci`; GitHub Rust CI
- Docs touched: `docs/jam_recipes.md`, `docs/execution_roadmap.md`
- Follow-ups: `None`

## Why This Ticket Existed

Fold the Recipe 15 real-source Feral-grid auto/fallback proof into the phase-level P012 all-lane output gate so one command proves both the existing observer/audio path and the current cautious auto timing contract.

## What Shipped

- Added recipe15-feral-grid-auto-proof to p012-all-lane-source-grid-output-proof.
- Documented the phase-level P012 all-lane proof from Recipe 15.
- Updated the roadmap deliverable note so P012's all-lane gate explicitly includes the real-source Feral-grid auto/fallback contract.

## Notes

- None
