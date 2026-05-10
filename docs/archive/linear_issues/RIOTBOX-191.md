# `RIOTBOX-191` Surface source-backed raw audition readiness in TUI cues

- Ticket: `RIOTBOX-191`
- Title: `Surface source-backed raw audition readiness in TUI cues`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-191/surface-source-backed-raw-audition-readiness-in-tui-cues`
- Project: `P008 | Scene Brain`
- Milestone: `P008 | Scene Brain`
- Status: `Done`
- Created: `2026-04-25`
- Started: `2026-04-25`
- Finished: `2026-04-25`
- Deleted from Linear: `2026-04-25`
- Branch: `feature/riotbox-191-source-backed-raw-cue`
- Linear branch: `feature/riotbox-191-surface-source-backed-raw-audition-readiness-in-tui-cues`
- PR: `#181`
- Merge commit: `04eb33c`
- Labels: `Audio`, `ux`
- Follow-ups: `RIOTBOX-192`

## Why This Ticket Existed

After source-window raw W-30 audition became real, the TUI still did not tell the user whether `[o]` was backed by decoded source material or using the synthetic fallback. This made the new audio path hard to trust from the instrument surface.

## What Shipped

- Added compact `audition raw/src` and `audition raw/fallback` Jam cues.
- Added Capture-screen readiness text for raw audition only, avoiding noisy `n/a` diagnostics in unrelated W-30 modes.
- Kept the Log W-30 lane compact with a shorter `prev raw/src` or `prev raw/fallback` label so mix diagnostics remain visible.
- Updated W-30 shell fixture expectations and jam recipe notes for the new cue.

## Verification

- `cargo fmt --all --check`
- `cargo test -p riotbox-app committed_raw_capture_audition_surfaces_source_fallback_readiness`
- `cargo test -p riotbox-app source_backed_raw_capture_audition_compact_label_uses_src_cue`
- `cargo test -p riotbox-app w30_fixture_backed_shell_regressions_hold`
- `cargo test`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `git diff --check`
- `just ci`
- GitHub Actions `rust-ci`

## Notes

- No manual live listening was required for this UI/readiness cue slice.
- This does not add waveform UI or a full sampler engine.
