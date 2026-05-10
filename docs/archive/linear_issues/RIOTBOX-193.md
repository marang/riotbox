# `RIOTBOX-193` Surface source-backed promoted W-30 preview readiness in TUI cues

- Ticket: `RIOTBOX-193`
- Title: `Surface source-backed promoted W-30 preview readiness in TUI cues`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-193/surface-source-backed-promoted-w-30-preview-readiness-in-tui-cues`
- Project: `P008 | Scene Brain`
- Milestone: `P008 | Scene Brain`
- Status: `Done`
- Created: `2026-04-25`
- Started: `2026-04-25`
- Finished: `2026-04-25`
- Deleted from Linear: `2026-04-25`
- Branch: `feature/riotbox-193-promoted-source-readiness`
- Linear branch: `feature/riotbox-193-surface-source-backed-promoted-w-30-preview-readiness-in-tui`
- PR: `#183`
- Merge commit: `750b276`
- Labels: `Audio`, `ux`
- Follow-ups: `RIOTBOX-194`

## Why This Ticket Existed

`RIOTBOX-192` made promoted audition and live recall / trigger previews source-backed when possible, but the TUI still needed to make that visible without adding more dense operator text.

## What Shipped

- Extended W-30 compact preview labels with `src` / `fallback` for promoted audition and live recall paths.
- Kept Log labels shorter with `audition/src`, `audition/fallback`, `recall/src`, and `recall/fallback` forms.
- Added non-raw Capture routing readiness text only when source-backed material is actually present, avoiding noisy fallback rows that push out routing diagnostics.
- Updated W-30 shell fixture expectations and generalized jam recipe wording around `.../src` and `.../fallback`.

## Verification

- `cargo fmt --all --check`
- `cargo test -p riotbox-app source_backed_promoted_and_recall_compact_labels_use_src_cue`
- `cargo test -p riotbox-app source_backed_raw_capture_audition_compact_label_uses_src_cue`
- `cargo test -p riotbox-app w30_fixture_backed_shell_regressions_hold`
- `cargo test`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `git diff --check`
- `just ci`
- GitHub Actions `rust-ci`

## Notes

- This is UI/readiness only and depends on the source-backed audio routing already shipped in `RIOTBOX-192`.
- It does not add new panels, waveform UI, or sampler-engine behavior.
