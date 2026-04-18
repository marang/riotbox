# `RIOTBOX-75` Surface W-30 bank-manager and damage-profile diagnostics in the shell

- Ticket: `RIOTBOX-75`
- Title: `Surface W-30 bank-manager and damage-profile diagnostics in the shell`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-75/surface-w-30-bank-manager-and-damage-profile-diagnostics-in-the-shell`
- Project: `P007 | W-30 MVP`
- Milestone: `W-30 MVP`
- Status: `Done`
- Created: `2026-04-17`
- Started: `2026-04-17`
- Finished: `2026-04-17`
- Branch: `feature/riotbox-75-w30-shell-diagnostics`
- Linear branch: `feature/riotbox-75-surface-w-30-bank-manager-and-damage-profile-diagnostics-in`
- Assignee: `Markus`
- Labels: `None`
- PR: `#69`
- Merge commit: `71e06e36cfb6d61346586f9700aeb58b9be6b957`
- Deleted from Linear: `Not deleted`
- Verification: `cargo fmt --all --check`, `cargo test`, `cargo clippy --all-targets --all-features -- -D warnings`, GitHub Actions `Rust CI` run `#188`
- Docs touched: `docs/research_decision_log.md`, `docs/README.md`, `docs/screenshots/w30_bank_forge_diagnostics_baseline.txt`
- Follow-ups: `RIOTBOX-76`

## Why This Ticket Existed

The W-30 MVP already had honest bank-swap and pad-forge controls on the committed preview seam, but the shell still made those new states too hard to read. Operators needed explicit bank-manager and damage-profile diagnostics in the same Jam, Capture, and Log surfaces rather than a second W-30 debug page or a forge-only screen.

## What Shipped

- surfaced compact W-30 bank-manager status in the Jam lanes panel next to the existing cue summary
- surfaced explicit pad-forge and bank-target diagnostics in the Capture routing panel without losing existing lineage context
- compacted the Log `W-30 Lane` panel so committed bank-swap and damage-profile state stay visible alongside mix and lineage diagnostics
- added fixture-backed shell regressions for committed bank-swap plus damage-profile state
- added the normalized review artifact at `docs/screenshots/w30_bank_forge_diagnostics_baseline.txt`

## Notes

- this slice is intentionally presentation-only on top of the already shipped W-30 bank-swap and pad-forge controls
- deeper forge behavior and any future W-30 editor surface remain later work on the same committed preview seam
