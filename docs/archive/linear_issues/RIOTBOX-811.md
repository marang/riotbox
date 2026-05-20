# `RIOTBOX-811` Surface MC-202 source-grid proof in observer/audio correlation

- Ticket: `RIOTBOX-811`
- Title: `Surface MC-202 source-grid proof in observer/audio correlation`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-811/surface-mc-202-source-grid-proof-in-observeraudio-correlation`
- Project: `P013 | All-Lane Musical Depth`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-20`
- Started: `2026-05-20`
- Finished: `2026-05-20`
- Branch: `feature/riotbox-811-mc202-observer-correlation`
- Linear branch: `feature/riotbox-811-surface-mc-202-source-grid-proof-in-observeraudio`
- Assignee: `Markus`
- Labels: `Audio`, `benchmark`
- PR: `#806 (https://github.com/marang/riotbox/pull/806)`
- Merge commit: `72e0870a811d9dd481dc778fc6310c20f36e35a0`
- Verification: `cargo fmt --check`; `git diff --check`; `python3 -m py_compile scripts/validate_observer_audio_summary_json.py scripts/validate_listening_manifest_json.py`; `cargo test -p riotbox-app --bin observer_audio_correlate`; `just observer-audio-summary-validator-fixtures`; `just syncopated-source-showcase-smoke`; `scripts/correlate_generated_feral_grid_observer.sh`; `cargo clippy -p riotbox-app --all-targets --all-features -- -D warnings`; `just audio-qa-ci`; `just ci`; GitHub Rust CI run `1957`
- Docs touched: `None`
- Follow-ups: `None`

## Why This Ticket Existed

RIOTBOX-810 added MC-202 source-grid proof to the representative showcase manifest/report, but observer/audio correlation still exposed only aggregate, TR-909, and W-30 source-grid alignment.

## What Shipped

- Added MC-202 source-grid alignment to observer/audio markdown and JSON summaries, required evidence checks, validator fixtures, syncopated smoke assertions, and merged PR #806.

## Notes

- Linear deletion was not performed because `LINEAR_API_TOKEN` was not present in this environment.
