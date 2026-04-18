# `RIOTBOX-124` Prototype a tiny visual countdown cue for queued scene actions

- Ticket: `RIOTBOX-124`
- Title: `Prototype a tiny visual countdown cue for queued scene actions`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-124/prototype-a-tiny-visual-countdown-cue-for-queued-scene-actions`
- Project: `P008 | Scene Brain`
- Milestone: `Scene Brain`
- Status: `Done`
- Created: `2026-04-18`
- Started: `2026-04-18`
- Finished: `2026-04-18`
- Branch: `feature/riotbox-124-scene-countdown-cue`
- Linear branch: `feature/riotbox-124-prototype-a-tiny-visual-countdown-cue-for-queued-scene`
- Assignee: `Markus`
- Labels: `None`
- PR: `#120`
- Merge commit: `1285adde76a638dadfd8f6d464a8d36b07f75506`
- Deleted from Linear: `Not deleted`
- Verification: `cargo fmt --all --check`, `cargo test`, `cargo clippy --all-targets --all-features -- -D warnings`, branch diff review, GitHub Actions `Rust CI` run `#329`
- Docs touched: `None`
- Follow-ups: `RIOTBOX-129`

## Why This Ticket Existed

The Scene Brain timing path had already become readable, but it was still almost entirely text-driven. Riotbox needed one tiny, removable visual cue that suggests bar proximity for queued scene actions without introducing a heavier timing widget.

## What Shipped

- added a four-step ASCII countdown cue inside the existing Scene Brain pulse line on `Jam`
- kept the cue bounded to the current timing seam instead of expanding into a separate timing panel
- refreshed focused shell expectations so the tiny countdown remains covered by the existing regression path

## Notes

- the cue is intentionally small and ASCII-first; later readability work can still replace it if a stronger timing widget proves necessary
- this slice was experimental UI refinement only and did not change queued action semantics
