# `RIOTBOX-52` Ticket Archive

- Ticket: `RIOTBOX-52`
- Title: `Surface MC-202 phrase and role diagnostics in Jam and Log screens`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-52/surface-mc-202-phrase-and-role-diagnostics-in-jam-and-log-screens`
- Project: `Riotbox MVP Buildout`
- Milestone: `MC-202 MVP`
- Status: `Done`
- Created: `2026-04-15`
- Started: `2026-04-15`
- Finished: `2026-04-15`
- Branch: `feature/riotbox-52-mc202-diagnostics`
- Linear branch: `feature/riotbox-52-surface-mc-202-phrase-and-role-diagnostics-in-jam-and-log`
- Assignee: `Markus`
- Labels: `None`
- PR: `#44`
- Merge commit: `cbdddb4`
- Deleted from Linear: `not deleted yet`
- Verification: `cargo fmt --all --check`, `cargo test`, `cargo clippy --all-targets --all-features -- -D warnings`
- Docs touched: `docs/research_decision_log.md`
- Follow-ups: `RIOTBOX-53`

## Why This Ticket Existed

`RIOTBOX-51` added the first honest follower-generation path, but the shell still did not answer the operator question of what the MC-202 lane believed it was doing. The next bounded slice needed to improve visibility in the normal `Jam` and `Log` surfaces without opening a dedicated synth inspector or a parallel debug route.

## What Shipped

- deepened the `Jam` lane summary with clearer MC-202 phrase, touch, and diagnostic cues
- added a dedicated `MC-202 Lane` diagnostics panel to the `Log` screen
- kept the slice read-only on top of existing lane state, pending-action intent, and action-log history
- updated the footer action hint so the visible shell now advertises the follower-generation key
- recorded the read-only diagnostics decision in `docs/research_decision_log.md`

## Notes

- The branch-level review did not surface any blocking issues. The only adjustment during the slice was relaxing a few width-coupled snapshot assertions after the new MC-202 panel narrowed the summary columns.
- GitHub's combined-status endpoint still returned an empty status set in this environment, so merge used explicit local green verification plus a mergeable PR.
