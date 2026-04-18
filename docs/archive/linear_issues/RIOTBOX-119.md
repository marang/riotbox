# `RIOTBOX-119` Add compact scene commit pulse cue on Jam for beat/bar/phrase timing

- Ticket: `RIOTBOX-119`
- Title: `Add compact scene commit pulse cue on Jam for beat/bar/phrase timing`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-119/add-compact-scene-commit-pulse-cue-on-jam-for-beatbarphrase-timing`
- Project: `P008 | Scene Brain`
- Milestone: `Scene Brain`
- Status: `Done`
- Created: `2026-04-18`
- Started: `2026-04-18`
- Finished: `2026-04-18`
- Branch: `feature/riotbox-119-scene-commit-pulse`
- Linear branch: `feature/riotbox-119-add-compact-scene-commit-pulse-cue-on-jam-for-beatbarphrase`
- Assignee: `Markus`
- Labels: `None`
- PR: `#112`
- Merge commit: `35881ce2e5804a03c4013961dd51494e1d5c18bb`
- Deleted from Linear: `Not deleted`
- Verification: `cargo fmt --all --check`, `cargo test`, `cargo clippy --all-targets --all-features -- -D warnings`, branch diff review, GitHub Actions `Rust CI` run `#316`
- Docs touched: `None`
- Follow-ups: `RIOTBOX-120`, `RIOTBOX-121`, `RIOTBOX-124`

## Why This Ticket Existed

After the Jam shell started telling the player that a scene action lands on `next bar`, the next missing piece was current timing context. Riotbox still asked the player to mentally align that boundary with the current beat / bar / phrase position instead of surfacing it directly on the same Jam surface.

## What Shipped

- added a compact scene-only pulse cue that shows current beat, bar, and phrase indices plus the pending scene boundary
- kept the pulse off when no scene action is queued so the perform shell stays quiet by default
- extended the scene-brain shell regressions so the pulse wording stays visible in both queued jump and queued restore paths

## Notes

- this remained a textual pulse cue, not a graphical countdown bar
- the cue intentionally reused the already available runtime transport state instead of introducing a second timing model
