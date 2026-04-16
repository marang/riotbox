# `RIOTBOX-54` Ticket Archive

- Ticket: `RIOTBOX-54`
- Title: `Add bounded W-30 promoted-material audition cue`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-54/add-bounded-w-30-promoted-material-audition-cue`
- Project: `Riotbox MVP Buildout`
- Milestone: `W-30 MVP`
- Status: `Done`
- Created: `2026-04-16`
- Started: `2026-04-16`
- Finished: `2026-04-16`
- Branch: `feature/riotbox-54-w30-audition-cue`
- Linear branch: `feature/riotbox-54-add-bounded-w-30-promoted-material-audition-cue`
- Assignee: `Markus`
- Labels: `None`
- PR: `#50`
- Merge commit: `c5309b3`
- Deleted from Linear: `not deleted yet`
- Verification: `cargo fmt --all --check`, `cargo test`, `cargo clippy --all-targets --all-features -- -D warnings`, `branch-level code-review`
- Docs touched: `docs/specs/action_lexicon_spec.md`, `docs/research_decision_log.md`
- Follow-ups: `RIOTBOX-57`, `RIOTBOX-58`, `RIOTBOX-59`

## Why This Ticket Existed

The W-30 MVP already had a replay-safe capture, promotion, and live-recall seam, but promoted material still behaved more like static bookkeeping than an instrument. Riotbox needed one more bounded next-bar cue that let promoted material be auditioned without opening a second sample browser, pad editor, or callback-only preview path.

## What Shipped

- Added bounded `w30.audition_promoted` on the existing `NextBar` W-30 pad seam.
- Blocked conflicting pending W-30 pad cues so audition and recall stay on one honest queue path.
- Surfaced pending audition intent in the shared shell summary and capture cue path.
- Committed audition through the same W-30 lane-focus seam and added a bounded `w30_grit` bump plus explicit action result.
- Added app and shell regression coverage for the new W-30 audition path.

## Notes

- This slice intentionally stopped at a queueable audition cue and did not open an audio-only preview path or deeper pad-editing surface.
- Later W-30 work should keep extending the same committed cue seam for diagnostics, regression coverage, and audio-facing preview behavior.
