# `RIOTBOX-284` Add first W-30 listening-pack manifest convention

- Ticket: `RIOTBOX-284`
- Title: `Add first W-30 listening-pack manifest convention`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-284/add-first-w-30-listening-pack-manifest-convention`
- Project: `P007 | W-30 MVP`
- Milestone: `P007 | W-30 MVP`
- Status: `Done`
- Created: `2026-04-26`
- Started: `2026-04-26`
- Finished: `2026-04-26`
- Deleted from Linear: `2026-04-26`
- Branch: `feature/riotbox-284-add-first-w-30-listening-pack-manifest-convention`
- Linear branch: `feature/riotbox-284-add-first-w-30-listening-pack-manifest-convention`
- PR: `#274`
- Merge commit: `545016c`
- Labels: `review-followup`, `benchmark`
- Follow-ups: `RIOTBOX-285`

## Why This Ticket Existed

`RIOTBOX-283` opened a first local W-30 preview WAV render helper. The helper needed a stable listening-pack convention so humans and agents can use the same case ID, output path, and notes shape without claiming a full generated listening-pack system.

## What Shipped

- Added the first local-only W-30 preview smoke listening-pack convention.
- Defined the stable case ID, output directory shape, render command, and human notes template.
- Ignored generated `artifacts/audio_qa/` outputs so local WAV/metrics files are not committed accidentally.
- Updated benchmark and audio QA docs to reference the convention.

## Verification

- `git diff --check`
- `just ci`
- GitHub Actions `rust-ci`

## Notes

- Docs/ops convention only; no baseline-vs-candidate comparison engine, generated listening-pack runner, or CI audio artifact gate changed.
- The fast-forward merge means the feature commit is also the merge commit on `main`.
