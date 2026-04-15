# `RIOTBOX-51` Ticket Archive

- Ticket: `RIOTBOX-51`
- Title: `Add first MC-202 follower generation action`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-51/add-first-mc-202-follower-generation-action`
- Project: `Riotbox MVP Buildout`
- Milestone: `MC-202 MVP`
- Status: `Done`
- Created: `2026-04-15`
- Started: `2026-04-15`
- Finished: `2026-04-15`
- Branch: `feature/riotbox-51-mc202-follower-entry`
- Linear branch: `feature/riotbox-51-add-first-mc-202-follower-generation-action`
- Assignee: `Markus`
- Labels: `None`
- PR: `#43`
- Merge commit: `de4d45c`
- Deleted from Linear: `not deleted yet`
- Verification: `cargo fmt --all --check`, `cargo test`, `cargo clippy --all-targets --all-features -- -D warnings`
- Docs touched: `docs/research_decision_log.md`
- Follow-ups: `RIOTBOX-52`, `RIOTBOX-53`

## Why This Ticket Existed

`RIOTBOX-48` made the MC-202 lane visible through a committed role toggle, but the milestone still lacked a real follower-line path. The next honest slice had to create usable follower-generation intent on the same phrase-boundary seam without pretending a full MC-202 engine or answer editor already existed.

## What Shipped

- added the first queueable `mc202.generate_follower` action on the existing `NextPhrase` seam
- committed follower generation now updates `mc202.role`, `mc202.phrase_ref`, and `mc202_touch`
- surfaced a lane-level pending follower-generation cue in the Jam shell
- blocked conflicting pending MC-202 phrase controls so the lane cannot queue contradictory same-boundary intent
- added app, shell, and core-view tests for the new MC-202 follower path
- recorded the bounded follower-generation decision in `docs/research_decision_log.md`

## Notes

- The branch-level review found one real issue before merge: `mc202.set_role` and `mc202.generate_follower` could both be queued against the same phrase boundary. That was fixed on-branch by blocking conflicting pending MC-202 phrase controls.
- GitHub's combined-status endpoint still returned an empty status set in this environment, so merge used explicit local green verification plus a mergeable PR.
