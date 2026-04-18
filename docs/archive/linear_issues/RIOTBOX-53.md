# `RIOTBOX-53` Ticket Archive

- Ticket: `RIOTBOX-53`
- Title: `Add fixture-backed MC-202 role and follower regression coverage`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-53/add-fixture-backed-mc-202-role-and-follower-regression-coverage`
- Project: `P006 | MC-202 MVP`
- Milestone: `MC-202 MVP`
- Status: `Done`
- Created: `2026-04-15`
- Started: `2026-04-16`
- Finished: `2026-04-16`
- Branch: `feature/riotbox-53-mc202-regression`
- Linear branch: `feature/riotbox-53-add-fixture-backed-mc-202-role-and-follower-regression`
- Assignee: `Markus`
- Labels: `None`
- PR: `#45`
- Merge commit: `5947922`
- Deleted from Linear: `not deleted yet`
- Verification: `cargo fmt --all --check`, `cargo test -p riotbox-app mc202_fixture_backed -- --nocapture`, `cargo test`, `cargo clippy --all-targets --all-features -- -D warnings`
- Docs touched: `Riotbox Project Updates`
- Follow-ups: `RIOTBOX-49`, `RIOTBOX-50`

## Why This Ticket Existed

`RIOTBOX-51` and `RIOTBOX-52` shipped the first real MC-202 role and follower-generation behavior plus deeper shell diagnostics, but the lane still lacked the replay-safe regression net already established on the TR-909 side. The next bounded slice needed to lock the current committed MC-202 behavior down before deeper phrase work or a lane switch reopened the seam.

## What Shipped

- added a shared MC-202 regression fixture corpus for role-toggle and follower-generation cases
- asserted committed lane state, `mc202.phrase_ref`, `mc202_touch`, and action result summaries at the app layer
- roundtripped committed session state through JSON save/load to keep the slice replay-safe
- asserted stable Jam and Log shell-visible MC-202 output from the same fixture corpus instead of relying on noisy full snapshots

## Notes

- The branch-level review did not surface blocking correctness or architecture findings after the shell helper was tightened to start from a clean queue/log state instead of the demo shell’s preloaded history.
- GitHub’s combined-status endpoint still returned an empty status set in this environment, so merge used explicit local green verification plus a mergeable PR.
