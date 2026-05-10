# `RIOTBOX-190` Extend W-30 fixtures for source-window raw audition playback

- Ticket: `RIOTBOX-190`
- Title: `Extend W-30 fixtures for source-window raw audition playback`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-190/extend-w-30-fixtures-for-source-window-raw-audition-playback`
- Project: `P008 | Scene Brain`
- Milestone: `P008 | Scene Brain`
- Status: `Done`
- Created: `2026-04-25`
- Started: `2026-04-25`
- Finished: `2026-04-25`
- Deleted from Linear: `2026-04-25`
- Branch: `feature/riotbox-190-source-window-w30-fixtures`
- Linear branch: `feature/riotbox-190-extend-w-30-fixtures-for-source-window-raw-audition-playback`
- PR: `#180`
- Merge commit: `6ce8f81`
- Labels: `Audio`, `benchmark`
- Follow-ups: `RIOTBOX-191`

## Why This Ticket Existed

`RIOTBOX-189` wired source-window samples into raw W-30 audition playback, but that behavior needed fixture-backed regression coverage so future audio changes cannot silently fall back to the synthetic preview path.

## What Shipped

- Extended the W-30 preview audio fixture schema with optional source-window preview data.
- Added a source-backed raw-capture audition fixture case using a deterministic ramp payload.
- Added signed-sum bounds so the fixture verifies the source-window payload is actually rendered.
- Kept the fixture self-contained without committing external audio binaries.

## Verification

- `cargo test -p riotbox-audio fixture_backed_w30_preview_audio_regressions_hold`
- `cargo test`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `git diff --check`
- `just ci`
- GitHub Actions `rust-ci`

## Notes

- No live manual listening was required for this fixture-only hardening slice.
- The formal offline listening-pack workflow from `docs/specs/audio_qa_workflow_spec.md` is still not operational in the repo.
