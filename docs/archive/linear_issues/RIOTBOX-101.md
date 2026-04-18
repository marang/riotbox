# `RIOTBOX-101` Tighten inspect-mode guidance and exit cues on the Jam shell

- Ticket: `RIOTBOX-101`
- Title: `Tighten inspect-mode guidance and exit cues on the Jam shell`
- Linear issue: `https://linear.app/riotbox/issue/riotbox-101/tighten-inspect-mode-guidance-and-exit-cues-on-the-jam-shell`
- Project: `P004 | Jam-First Playable Slice`
- Milestone: `Jam-First Playable Slice`
- Status: `Done`
- Created: `2026-04-17`
- Started: `2026-04-17`
- Finished: `2026-04-18`
- Branch: `feature/riotbox-101-inspect-guidance`
- Linear branch: `feature/riotbox-101-tighten-inspect-mode-guidance-and-exit-cues-on-the-jam-shell`
- Assignee: `Markus`
- Labels: `None`
- PR: `#95`
- Merge commit: `f1a8c9205e69a5d18bac01b3ee93de983ef9e3c5`
- Deleted from Linear: `Not deleted`
- Verification: `cargo fmt --all --check`, `cargo test`, `cargo clippy --all-targets --all-features -- -D warnings`, GitHub Actions `Rust CI` run `#270`
- Docs touched: `None`
- Follow-ups: `RIOTBOX-103`, `RIOTBOX-104`, `RIOTBOX-105`, `RIOTBOX-106`

## Why This Ticket Existed

`RIOTBOX-97` added a bounded Jam inspect mode, but the first follow-up friction was still mode confusion: new players could open inspect without a strong explanation of what it was for, and once inside it was not explicit enough that the surface was read-only or how to get back to perform mode quickly. Riotbox needed one small pass that clarified inspect without adding more inspect density.

## What Shipped

- rewrote the Jam inspect enter/exit status messages so the shell now explains both the purpose and the return path
- changed the Jam tab purpose text to describe inspect as a read-only surface
- made the footer advertise `i` as `return to perform` while inspect is active
- updated the help overlay to frame inspect as a bounded detour from Jam perform mode instead of a generic mode toggle
- refreshed UI tests for the new inspect wording and first-run gating behavior

## Notes

- this slice stayed strictly presentation-only; it changed no action semantics, queue logic, or inspect data density
- the footer now changes its second line while inspect is active so the shell does not keep advertising queueable gestures on a read-only surface
- the next honest follow-ups are the broader perform-surface prioritization passes: pending-action reduction, primary-vs-advanced gesture grouping, lane-card compression, and clearer post-commit next-step cues
