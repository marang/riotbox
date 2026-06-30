# `RIOTBOX-1331` P023: Make TR-909 a source-derived differentiated drum lane

- Ticket: `RIOTBOX-1331`
- Title: `P023: Make TR-909 a source-derived differentiated drum lane`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1331/p023-make-tr-909-a-source-derived-differentiated-drum-lane`
- Project: `None`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-29`
- Started: `2026-06-29`
- Finished: `2026-06-30`
- Branch: `feature/riotbox-1331-p023-make-tr-909-a-source-derived-differentiated-drum-lane`
- Linear branch: `feature/riotbox-1331-p023-make-tr-909-a-source-derived-differentiated-drum-lane`
- Assignee: `Markus`
- Labels: `Audio`
- PR: `#1304 (https://github.com/marang/riotbox/pull/1304)`
- Merge commit: `bf380f34fd7e74ae9fdd46c794766bafaa55f399`
- Deleted from Linear: `2026-06-30`
- Verification: `cargo fmt --check; cargo test -p riotbox-audio; cargo test -p riotbox-audio --bin feral_grid_pack; cargo test -p riotbox-app; cargo clippy -p riotbox-audio -p riotbox-app --all-targets --all-features -- -D warnings; just ci; GitHub rust-ci pass`
- Docs touched: `None`
- Follow-ups: `None`

## Why This Ticket Existed

TR-909 source-support profiles now alter audible runtime voice balance instead of only metadata, so source roles can drive distinct drum support.

## What Shipped

- Runtime TR-909 waveform blending now separates kick/snare/hat balance by source profile, role, pattern adoption, and phrase variation; generated-support mix and kick-pressure policies were tuned; offline profile/role regression tests and full CI are green.

## Notes

- Automated professional/audio QA is green; structured human listening verdict remains human_verdict: unverified, so this is not claimed as musical-pass or release-ready quality proof.
