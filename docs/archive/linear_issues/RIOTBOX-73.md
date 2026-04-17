# `RIOTBOX-73` Add first explicit W-30 bank-swap control on the committed focus seam

- Ticket: `RIOTBOX-73`
- Title: `Add first explicit W-30 bank-swap control on the committed focus seam`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-73/add-first-explicit-w-30-bank-swap-control-on-the-committed-focus-seam`
- Project: `Riotbox MVP Buildout`
- Milestone: `W-30 MVP`
- Status: `Done`
- Created: `2026-04-17`
- Started: `2026-04-17`
- Finished: `2026-04-17`
- Branch: `feature/riotbox-73-w30-bank-swap-control`
- Linear branch: `feature/riotbox-73-add-first-explicit-w-30-bank-swap-control-on-the-committed`
- Assignee: `Markus`
- Labels: `None`
- PR: `#67`
- Merge commit: `ad4904ca8a89e163920b97b3947f56eb05668b8e`
- Deleted from Linear: `Not deleted`
- Verification: `cargo fmt --all --check`, `cargo test`, `cargo clippy --all-targets --all-features -- -D warnings`, GitHub Actions `Rust CI` run `#182`
- Docs touched: `docs/research_decision_log.md`
- Follow-ups: `RIOTBOX-74`, `RIOTBOX-75`, `RIOTBOX-76`

## Why This Ticket Existed

`RIOTBOX-72` split live recall onto explicit `w30.live_recall`, which freed `w30.swap_bank` for the first honest bank-manager control on the same W-30 preview seam. The repo still needed one real committed bank movement path before any broader bank-grid or pad-forge work could land without reopening misleading action semantics.

## What Shipped

- added the first real `w30.swap_bank` queue path on the existing committed W-30 preview seam
- resolved bank swaps from actual promoted W-30 targets, rotating to the next bank, preserving the focused pad when possible, and falling back to the first promoted pad in that bank otherwise
- blocked bank swaps against other pending W-30 pad cues and committed the same focused-bank, focused-pad, and preview-facing lane updates through the existing seam
- surfaced explicit pending and committed bank-swap cues in the core Jam view model, Capture shell, and event loop messaging
- added queue, commit, and shell regression coverage for the new bank-manager path

## Notes

- this slice stays intentionally bounded to promoted-bank rotation only; it does not open a full bank-grid editor, empty-bank travel, or deeper pad-forge controls
- the W-30 MVP now has honest separation between `w30.live_recall`, `w30.step_focus`, and `w30.swap_bank` on the same replay-safe committed seam
