# `RIOTBOX-286` Add W-30 preview smoke pack output flags

- Ticket: `RIOTBOX-286`
- Title: `Add W-30 preview smoke pack output flags`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-286/add-w-30-preview-smoke-pack-output-flags`
- Project: `P000 | Repo Ops / QA / Workflow`
- Milestone: `P000 | Repo Ops / QA / Workflow`
- Status: `Done`
- Created: `2026-04-26`
- Started: `2026-04-26`
- Finished: `2026-04-26`
- Deleted from Linear: `2026-04-26`
- Branch: `feature/riotbox-286-add-w-30-preview-smoke-pack-output-flags`
- Linear branch: `feature/riotbox-286-add-w-30-preview-smoke-pack-output-flags`
- PR: `#276`
- Merge commit: `ab9e2c2`
- Labels: `workflow`, `benchmark`
- Follow-ups: `RIOTBOX-287`

## Why This Ticket Existed

`RIOTBOX-285` defined the baseline-vs-candidate artifact convention, but the W-30 preview render helper still required callers to hand-type the full output path. The helper needed small flags to derive the convention path directly while preserving explicit output overrides for ad hoc renders.

## What Shipped

- Added `--date` and `--role baseline|candidate` flags to `w30_preview_render`.
- Preserved `--out PATH` as an explicit override for ad hoc paths.
- Updated W-30 smoke pack and audio QA artifact convention docs to use the new flags.

## Verification

- `cargo test -p riotbox-audio --bin w30_preview_render`
- `cargo run -p riotbox-audio --bin w30_preview_render -- --date 2026-04-26 --role baseline --duration-seconds 0.1`
- `git diff --check`
- `just ci`
- GitHub Actions `rust-ci`

## Notes

- Local helper ergonomics only; no baseline lookup, comparison engine, generated pack runner, or CI audio artifact gate changed.
- The fast-forward merge means the feature commit is also the merge commit on `main`.
