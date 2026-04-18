# `RIOTBOX-97` Add deeper Jam inspect surface without re-bloating the perform-first shell

- Ticket: `RIOTBOX-97`
- Title: `Add deeper Jam inspect surface without re-bloating the perform-first shell`
- Linear issue: `https://linear.app/riotbox/issue/riotbox-97/add-deeper-jam-inspect-surface-without-re-bloating-the-perform-first`
- Project: `P004 | Jam-First Playable Slice`
- Milestone: `None`
- Status: `Done`
- Created: `2026-04-17`
- Started: `2026-04-17`
- Finished: `2026-04-17`
- Branch: `feature/riotbox-97-jam-inspect`
- Linear branch: `feature/riotbox-97-add-deeper-jam-inspect-surface-without-re-bloating-the`
- Assignee: `Markus`
- Labels: `None`
- PR: `#92`
- Merge commit: `ecae33fee53156e46aba2d7e6e34b18cba8024a2`
- Deleted from Linear: `Not deleted`
- Verification: `cargo fmt --all`, `cargo test`, `cargo clippy --all-targets --all-features -- -D warnings`
- Docs touched: `docs/research_decision_log.md`, `docs/README.md`, `docs/screenshots/jam_inspect_mode_baseline.txt`
- Follow-ups: `RIOTBOX-101`, `RIOTBOX-102`

## Why This Ticket Existed

The perform-first Jam shell was already reduced and easier to read, but it still lacked a bounded way to look a little deeper without leaving `Jam` entirely or reopening the old dashboard density. Riotbox needed one confidence-building inspect layer that stayed inside the same shell spine and did not invent a second hidden operator surface.

## What Shipped

- added an explicit `perform / inspect` toggle inside the Jam shell
- kept `Now / Next / Trust` as the stable top frame while swapping the lower Jam panels into a deeper inspect view
- reused existing MC-202, W-30, TR-909, source, capture, and runtime seams instead of introducing a new model
- blocked inspect mode during the first-run onramp so the first guided move stays simple
- added focused shell snapshot and key-handling coverage plus a normalized inspect-mode baseline artifact

## Notes

- this slice was intentionally UX-only; it added no new action, audio, or persistence behavior
- the first rebase after `RIOTBOX-100` exposed one conflict in `docs/research_decision_log.md`, which was resolved by keeping both accepted decisions in order
- the next honest follow-up after inspect mode was not more inspect density, but clearer post-first-run learning paths, which became `RIOTBOX-102`

