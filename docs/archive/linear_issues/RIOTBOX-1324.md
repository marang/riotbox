# `RIOTBOX-1324` P023: Migrate audio MC-202 and source-audio include shells

- Ticket: `RIOTBOX-1324`
- Title: `P023: Migrate audio MC-202 and source-audio include shells`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1324/p023-migrate-audio-mc-202-and-source-audio-include-shells`
- Project: `None`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-29`
- Started: `2026-06-29`
- Finished: `2026-06-29`
- Branch: `feature/riotbox-1324-p023-migrate-audio-mc-202-and-source-audio-include-shells`
- Linear branch: `feature/riotbox-1324-p023-migrate-audio-mc-202-and-source-audio-include-shells`
- Assignee: `Markus`
- Labels: `Audio`
- PR: `#1298 (https://github.com/marang/riotbox/pull/1298)`
- Merge commit: `2153c20a6633777f95519e38a5a21303f826e368`
- Deleted from Linear: `2026-06-29`
- Verification: `cargo fmt --check; scripts/check_no_textual_includes.sh; git diff --check; cargo test -p riotbox-audio; cargo clippy -p riotbox-audio --all-targets --all-features -- -D warnings; cargo check -p riotbox-app; just mc202-real-source-listening-pack-smoke; just source-timing-wav-probe; GitHub rust-ci passed on PR #1298`
- Docs touched: `docs/engineering/textual_include_allowlist.txt; docs/engineering/textual_include_inventory_2026-06-29.md`
- Follow-ups: `Continue P023 with RIOTBOX-1325 and RIOTBOX-1337 for CLI module ownership.`

## Why This Ticket Existed

Audio MC-202 and source-audio still used textual include shells at render/cache seams; P023 needs clearer Rust ownership without changing DSP or WAV behavior.

## What Shipped

- Replaced MC-202 and source-audio root include shells with real module roots and compatibility re-exports.
- Kept MC-202 sound design internal while preserving render buffer API and source-audio cache API.
- Reduced textual include inventory from 243/19 to 236/17 and recorded both audio owners as migrated.

## Notes

- None
