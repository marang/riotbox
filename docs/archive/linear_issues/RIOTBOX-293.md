# `RIOTBOX-293` Make source-backed W-30 preview load PCM24 in the app path

- Ticket: `RIOTBOX-293`
- Title: `Make source-backed W-30 preview load PCM24 in the app path`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-293/make-source-backed-w-30-preview-load-pcm24-in-the-app-path`
- Project: `P007 | W-30 MVP`
- Milestone: `P007 | W-30 MVP`
- Status: `Done`
- Created: `2026-04-26`
- Started: `2026-04-26`
- Finished: `2026-04-26`
- Deleted from Linear: `2026-04-26`
- Branch: `feature/riotbox-293-make-source-backed-w-30-preview-load-pcm24-in-the-app-path`
- Linear branch: `feature/riotbox-293-make-source-backed-w-30-preview-load-pcm24-in-the-app-path`
- PR: `#283`
- Merge commit: `43352c4`
- Labels: `benchmark`
- Follow-ups: `RIOTBOX-294`

## Why This Ticket Existed

The offline W-30 smoke helper could render PCM16/PCM24 source windows, but the app load path still used a PCM16-specific loader name. That made the app-facing W-30 preview path look narrower than the decoder capability and could let the app diverge from the QA helper.

## What Shipped

- Renamed the source audio cache loader to the format-neutral `load_pcm_wav`.
- Updated the W-30 preview render helper and app persistence/load path to use the format-neutral loader.
- Added app-level coverage proving `from_json_files` loads a PCM24 source WAV into the source audio cache.
- Removed stale `load_pcm16_wav` naming across the workspace.

## Verification

- `cargo fmt --check`
- `cargo test -p riotbox-audio source_audio::tests`
- `cargo test -p riotbox-app loads_pcm24_source_audio_cache_from_app_files`
- `git diff --check`
- `just ci`
- Branch diff reviewed with the `code-review` skill
- GitHub Actions `rust-ci`

## Notes

- API naming and app-cache alignment slice only; no new decoder formats, TUI controls, pad sequencing, or generated audio artifact conventions changed.
- The fast-forward merge means the feature commit is also the merge commit on `main`.
