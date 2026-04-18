# `RIOTBOX-48` Ticket Archive

- Ticket: `RIOTBOX-48`
- Title: `Start MC-202 MVP with the first bounded lead or follower control`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-48/start-mc-202-mvp-with-the-first-bounded-lead-or-follower-control`
- Project: `P006 | MC-202 MVP`
- Milestone: `MC-202 MVP`
- Status: `Done`
- Created: `2026-04-15`
- Started: `2026-04-15`
- Finished: `2026-04-15`
- Branch: `feature/riotbox-48-mc202-entry`
- Linear branch: `feature/riotbox-48-start-mc-202-mvp-with-the-first-bounded-lead-or-follower`
- Assignee: `Markus`
- Labels: `None`
- PR: `#42`
- Merge commit: `24b5431`
- Deleted from Linear: `not deleted yet`
- Verification: `cargo fmt --all --check`, `cargo test`, `cargo clippy --all-targets --all-features -- -D warnings`
- Docs touched: `docs/research_decision_log.md`
- Follow-ups: `RIOTBOX-49`, `RIOTBOX-50`

## Why This Ticket Existed

Riotbox needed a real first MC-202 control so the roadmap could leave the TR-909 lane honestly, but the follower or answer generator was not ready yet. The smallest honest entry slice was a bounded role control that used the same queue and phrase-boundary seam as the rest of the product.

## What Shipped

- added the first queueable `mc202.set_role` action on the existing `NextPhrase` commit seam
- made commit-time side effects update `mc202.role`, a simple `mc202.phrase_ref`, and `mc202_touch`
- surfaced pending MC-202 role intent in `JamViewModel` and the Jam shell
- added app, shell, and core-view coverage for the new MC-202 role-control path
- recorded the bounded MC-202 entry decision in `docs/research_decision_log.md`

## Notes

- The slice intentionally stopped at committed role control. It did not try to fake follower generation, answer logic, or deeper synth behavior.
- GitHub's combined-status endpoint again returned an empty status set in this environment, and `gh pr checks` was unavailable because the local CLI was not authenticated, so merge used explicit local green verification plus a mergeable PR.
