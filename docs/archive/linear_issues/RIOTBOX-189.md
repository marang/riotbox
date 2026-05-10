# `RIOTBOX-189` Wire source-window captures into raw W-30 preview playback

- Ticket: `RIOTBOX-189`
- Title: `Wire source-window captures into raw W-30 preview playback`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-189/wire-source-window-captures-into-raw-w-30-preview-playback`
- Project: `P008 | Scene Brain`
- Milestone: `P008 | Scene Brain`
- Status: `Done`
- Created: `2026-04-25`
- Started: `2026-04-25`
- Finished: `2026-04-25`
- Deleted from Linear: `2026-04-25`
- Branch: `feature/riotbox-189-source-window-w30-preview`
- Linear branch: `feature/riotbox-189-wire-source-window-captures-into-raw-w-30-preview-playback`
- PR: `#179`
- Merge commit: `e0d67ca`
- Labels: `Audio`, `Core`
- Follow-ups: `RIOTBOX-190`

## Why This Ticket Existed

`RIOTBOX-187` made source audio decodable and `RIOTBOX-188` made direct captures remember their source window, but raw W-30 audition still used synthetic preview behavior. The next product-spine step was to bridge those contracts into audible raw-capture preview without adding a second sampler architecture.

## What Shipped

- Loaded the decoded source-audio cache during app session load when the source WAV path is available.
- Projected `CaptureRef.source_window` into a fixed-size W-30 preview sample payload for raw capture audition.
- Copied that payload into the audio callback through atomic state and used it for raw-capture preview rendering.
- Preserved the existing synthetic preview fallback when no source window or cache is available.
- Added app projection and audio-buffer tests for source-window raw audition behavior.
- Updated `docs/specs/audio_core_spec.md` to document the callback-safe preview excerpt seam and current limitations.

## Verification

- `cargo fmt --all --check`
- `cargo test -p riotbox-app raw_capture_audition_projects_source_window_preview_samples`
- `cargo test -p riotbox-audio w30_raw_capture_audition_uses_source_window_samples_when_available`
- `cargo test`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `git diff --check`
- `just ci`
- GitHub Actions `rust-ci`

## Notes

- No live manual listening was run from the sandboxed agent context.
- The formal offline listening-pack workflow from `docs/specs/audio_qa_workflow_spec.md` is still not operational in the repo.
- This is still a bounded preview excerpt, not a full pad-bank sample engine.
