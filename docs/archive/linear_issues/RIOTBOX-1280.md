# `RIOTBOX-1280` Remove W-30 synthetic preview fallback from musician-facing output

- Ticket: `RIOTBOX-1280`
- Title: `Remove W-30 synthetic preview fallback from musician-facing output`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1280/remove-w-30-synthetic-preview-fallback-from-musician-facing-output`
- Project: `P023 | Sound Excellence / Production Quality`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-15`
- Started: `2026-06-18`
- Finished: `2026-06-18`
- Branch: `feature/riotbox-1280-remove-w30-synthetic-preview-fallback`
- Linear branch: `feature/riotbox-1280-remove-w-30-synthetic-preview-fallback-from-musician-facing`
- Assignee: `Markus`
- Labels: `Audio`
- PR: `#1255 (https://github.com/marang/riotbox/pull/1255)`
- Merge commit: `bf88c6eb71353057585493b2b60e5fda474b42bf`
- Deleted from Linear: `2026-06-18`
- Verification: `cargo fmt; cargo test -p riotbox-core view::jam; cargo test -p riotbox-app w30_; cargo test -p riotbox-app; cargo test -p riotbox-audio --bin w30_preview_render --bin w30_preview_compare; cargo clippy --all-targets --all-features -- -D warnings; git diff --check; just audio-qa-ci; just ci before final UI-label review fix; GitHub rust-ci passed on PR #1255.`
- Docs touched: `docs/jam_recipes.md; docs/specs/audio_qa_workflow_spec.md`
- Follow-ups: `None`

## Why This Ticket Existed

Remove the W-30 synthetic preview fallback from musician-facing output paths so materialless previews surface as unavailable instead of playing placeholder sound.

## What Shipped

- W-30 preview routing is silent at zero music level without source-window samples or artifact-backed pad playback; Jam/Capture/Log cues now distinguish src, artifact, and unavailable; W-30 fixtures and docs now treat synthetic baselines as non-product diagnostic controls.

## Notes

- Branch review found and fixed a misleading W-30 Capture cue where source-window metadata alone could label a silent render path as src; the UI now respects current preview material for W-30 targets.
