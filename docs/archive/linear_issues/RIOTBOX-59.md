# `RIOTBOX-59` Ticket Archive

- Ticket: `RIOTBOX-59`
- Title: `Prepare audio-facing W-30 preview render seam`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-59/prepare-audio-facing-w-30-preview-render-seam`
- Project: `Riotbox MVP Buildout`
- Milestone: `W-30 MVP`
- Status: `Done`
- Created: `2026-04-16`
- Started: `2026-04-16`
- Finished: `2026-04-16`
- Branch: `feature/riotbox-59-w30-preview-render-seam`
- Linear branch: `feature/riotbox-59-prepare-audio-facing-w-30-preview-render-seam`
- Assignee: `Markus`
- Labels: `None`
- PR: `#53`
- Merge commit: `42989ff`
- Deleted from Linear: `not deleted yet`
- Verification: `cargo fmt --all`, `cargo test`, `cargo clippy --all-targets --all-features -- -D warnings`, `branch-level code-review`
- Docs touched: `docs/research_decision_log.md`
- Follow-ups: `RIOTBOX-60`, `RIOTBOX-61`

## Why This Ticket Existed

The W-30 MVP already had committed live recall, promoted-material audition, shell diagnostics, and replay-safe regression coverage, but it still lacked an honest audio-facing seam. Riotbox needed one bounded preview contract that could reach the audio runtime without inventing a second W-30 runtime model or pretending full sample playback was already solved.

## What Shipped

- Added a typed `W30PreviewRenderState` module in `riotbox-audio`.
- Added shared W-30 preview runtime storage and update plumbing alongside the existing TR-909 render seam.
- Derived W-30 preview mode, profile, target, and mix state from the committed session, capture, and action-log seam in `riotbox-app`.
- Surfaced the preview seam in current Jam, Capture, and Log diagnostics.
- Kept the shared W-30 regression corpus aligned with the tighter shell summaries.
- Recorded the seam boundary in the research decision log.

## Notes

- This slice deliberately stopped short of real W-30 sample playback.
- Later audible preview, pad playback, and resample taps should extend this same typed preview seam instead of creating a callback-only side path.
