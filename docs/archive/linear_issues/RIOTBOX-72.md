# `RIOTBOX-72` Split W-30 live recall from bank-swap semantics before bank-manager controls

- Ticket: `RIOTBOX-72`
- Title: `Split W-30 live recall from bank-swap semantics before bank-manager controls`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-72/split-w-30-live-recall-from-bank-swap-semantics-before-bank-manager`
- Project: `Riotbox MVP Buildout`
- Milestone: `W-30 MVP`
- Status: `Done`
- Created: `2026-04-17`
- Started: `2026-04-17`
- Finished: `2026-04-17`
- Branch: `feature/riotbox-72-w30-live-recall-command-split`
- Linear branch: `feature/riotbox-72-split-w-30-live-recall-from-bank-swap-semantics-before-bank`
- Assignee: `Markus`
- Labels: `None`
- PR: `#66`
- Merge commit: `39e8a4c1a397528d129c4f9fe5971f2dbecdc37b`
- Deleted from Linear: `Not deleted`
- Verification: `cargo fmt --all --check`, `cargo test`, `cargo clippy --all-targets --all-features -- -D warnings`, GitHub Actions `Rust CI` run `#179`
- Docs touched: `docs/specs/action_lexicon_spec.md`, `docs/research_decision_log.md`, `docs/screenshots/capture_w30_live_recall_baseline.txt`
- Follow-ups: `RIOTBOX-73`, `RIOTBOX-74`, `RIOTBOX-75`, `RIOTBOX-76`

## Why This Ticket Existed

The repo already had committed `w30.step_focus` behavior on the W-30 preview seam, but live recall was still recorded as `w30.swap_bank`. That made action history misleading and blocked the first honest bank-manager control from landing cleanly on the same seam.

## What Shipped

- added explicit `w30.live_recall` to the core action lexicon and action enum
- moved pending recall summaries, queue state, committed side effects, and shell labels onto `w30.live_recall`
- kept the shipped recall targeting, `NextBar` quantization, committed preview mode, and focused-pad updates unchanged
- updated the live-recall shell baseline and targeted queue/commit regressions for the new command name

## Notes

- `w30.swap_bank` remains reserved for future real bank-manager movement instead of continuing as a recall alias
- this was a semantic cleanup slice only; it did not ship any new bank-swap behavior
