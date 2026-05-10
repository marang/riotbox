# `RIOTBOX-237` Document Scene-target TR-909 accent audio baseline

- Ticket: `RIOTBOX-237`
- Title: `Document Scene-target TR-909 accent audio baseline`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-237/document-scene-target-tr-909-accent-audio-baseline`
- Project: `P008 | Scene Brain`
- Milestone: `P008 | Scene Brain`
- Status: `Done`
- Created: `2026-04-26`
- Started: `2026-04-26`
- Finished: `2026-04-26`
- Deleted from Linear: `2026-04-26`
- Branch: `feature/riotbox-237-scene-target-accent-baseline`
- Linear branch: `feature/riotbox-237-document-scene-target-tr-909-accent-audio-baseline`
- PR: `#227`
- Merge commit: `0ba4017`
- Labels: `benchmark`, `ux`
- Follow-ups: `RIOTBOX-238`

## Why This Ticket Existed

`RIOTBOX-236` added the first bounded audible Scene-target TR-909 support accent. The benchmark archive needed to state the current audio QA truth: buffer regression exists, but no formal listening-pack or real-session manual gate exists yet.

## What Shipped

- Added `docs/benchmarks/scene_tr909_support_accent_audio_baseline_2026-04-26.md`.
- Captured the `scene_target` versus `transport_bar` accent contract at buffer-regression level.
- Documented current limits around listening packs and manual real-session validation.
- Indexed the new baseline in `docs/benchmarks/README.md` and `docs/README.md`.

## Verification

- `git diff --check`
- `rg -n "scene_tr909_support_accent|scene_target_context_adds_bounded_support_accent|1.3x|listening-pack|support-accent" docs/benchmarks/scene_tr909_support_accent_audio_baseline_2026-04-26.md docs/benchmarks/README.md docs/README.md`
- `just ci`
- GitHub Actions `rust-ci`

## Notes

- Docs-only audio QA baseline slice; no runtime behavior, listening harness, or TUI wording changed.
- The fast-forward merge means the feature commit is also the merge commit on `main`.
