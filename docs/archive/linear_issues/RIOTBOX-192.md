# `RIOTBOX-192` Route promoted W-30 recall and trigger previews through source-window samples

- Ticket: `RIOTBOX-192`
- Title: `Route promoted W-30 recall and trigger previews through source-window samples`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-192/route-promoted-w-30-recall-and-trigger-previews-through-source-window`
- Project: `P008 | Scene Brain`
- Milestone: `P008 | Scene Brain`
- Status: `Done`
- Created: `2026-04-25`
- Started: `2026-04-25`
- Finished: `2026-04-25`
- Deleted from Linear: `2026-04-25`
- Branch: `feature/riotbox-192-promoted-source-window-preview`
- Linear branch: `feature/riotbox-192-route-promoted-w-30-recall-and-trigger-previews-through`
- PR: `#182`
- Merge commit: `5e1b7d0`
- Labels: `Audio`, `Core`
- Follow-ups: `RIOTBOX-193`

## Why This Ticket Existed

Raw W-30 audition was source-backed, but promoted audition and trigger/recall still used the synthetic W-30 preview voice even when the focused capture had source-window metadata. This left the first post-promotion hit detached from the captured source material.

## What Shipped

- Projected source-window preview samples for every non-idle W-30 preview mode when the focused capture has source material.
- Let the audio callback render the source-window payload for promoted audition and live recall / trigger preview paths.
- Preserved the synthetic fallback whenever no source-window payload is available.
- Added app projection coverage and audio-buffer tests for promoted audition and live recall.
- Updated `docs/specs/audio_core_spec.md` to describe source-backed W-30 preview paths beyond raw audition.

## Verification

- `cargo fmt --all --check`
- `cargo test -p riotbox-app promoted_and_recall_w30_previews_project_source_window_preview_samples`
- `cargo test -p riotbox-audio w30_promoted_audition_uses_source_window_samples_when_available`
- `cargo test -p riotbox-audio w30_live_recall_uses_source_window_samples_when_available`
- `cargo test`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `git diff --check`
- `just ci`
- GitHub Actions `rust-ci`

## Notes

- No manual live listening was run from the sandboxed agent context.
- This remains a bounded preview excerpt seam, not full pad-bank sample streaming.
