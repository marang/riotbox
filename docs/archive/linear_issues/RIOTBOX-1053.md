# `RIOTBOX-1053` Add P015 first-run next-move cue audit

- Ticket: `RIOTBOX-1053`
- Title: `Add P015 first-run next-move cue audit`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1053/add-p015-first-run-next-move-cue-audit`
- Project: `P015 | Productization Alpha`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-31`
- Started: `2026-05-31`
- Finished: `2026-05-31`
- Branch: `feature/riotbox-1053-p015-next-move-cue-audit`
- Linear branch: `feature/riotbox-1053-add-p015-first-run-next-move-cue-audit`
- Assignee: `Markus`
- Labels: None
- PR: `#1030 (https://github.com/marang/riotbox/pull/1030)`
- Merge commit: `6cb4bd9a0f4b837b2874fbc5296d707ef5732e1c`
- Deleted from Linear: `2026-05-31`
- Verification: `cargo fmt --check: pass`; `cargo test -p riotbox-app first_result_guidance -- --nocapture: pass`; `cargo test -p riotbox-app onramp -- --nocapture: pass`; `git diff --check: pass`; `targeted rg next-move language check: pass`; `just ci: pass`; `GitHub rust-ci on PR #1030: pass`
- Docs touched: `docs/screenshots/jam_first_30_seconds_baseline.txt`
- Follow-ups: `None`

## Why This Ticket Existed

P015 needed the first Jam minute audited so next-move guidance respects P012/P014 timing trust boundaries instead of making scene movement look equally safe under cautious timing.

## What Shipped

- Changed first-result Start Here next-move copy to derive from Arrangement / Scene readiness.
- Cautious, fallback, unknown, and not-enough-scene states now steer toward lane gestures before scene jump.
- Locked-grid state still promotes scene jump beside follow, with targeted UI coverage and first-30-seconds baseline update.

## Notes

- None
