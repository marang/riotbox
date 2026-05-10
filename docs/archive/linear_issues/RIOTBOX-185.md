# `RIOTBOX-185` Add a bounded raw-capture audition seam

- Ticket: `RIOTBOX-185`
- Title: `Add a bounded raw-capture audition seam`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-185/add-a-bounded-raw-capture-audition-seam`
- Project: `P008 | Scene Brain`
- Milestone: `P008 | Scene Brain`
- Status: `Done`
- Created: `2026-04-25`
- Started: `2026-04-25`
- Finished: `2026-04-25`
- Branch: `feature/riotbox-185-raw-capture-audition`
- Linear branch: `feature/riotbox-185-add-a-bounded-raw-capture-audition-seam`
- PR: `#175`
- Merge commit: `c24d079`
- Labels: `ux`, `TUI`
- Follow-ups: `RIOTBOX-186`

## Why This Ticket Existed

First-use feedback still showed a blind Capture gap: stored material was visible before the performer had a direct way to hear that raw captured moment. The repo already had W-30 preview plumbing, so the next honest slice was one replay-safe raw-capture audition path instead of more wording only.

## What Shipped

- Added `w30.audition_raw_capture` to the action lexicon and committed session preview state.
- Routed the existing `[o]` audition key through raw-capture audition before promotion, while keeping promoted audition behavior once a W-30 capture exists.
- Added a raw-capture W-30 preview render/source profile on the existing audio callback seam.
- Surfaced raw audition pending and committed cues in `Capture`, `Log`, and compact W-30 preview summaries.
- Updated Capture learning docs and readability baseline wording so stored captures now advertise `[o] raw` before `[p]->[w]`.

## Verification

- `cargo fmt --all --check`
- `cargo test`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `git diff --check`
- `just ci`
- GitHub Actions `rust-ci`

## Notes

- This does not implement real recorded-waveform playback yet. It intentionally deepens the existing W-30 preview seam with a distinct raw-audition state until the fuller sampler path lands.
