# `RIOTBOX-285` Define baseline-vs-candidate audio QA artifact convention

- Ticket: `RIOTBOX-285`
- Title: `Define baseline-vs-candidate audio QA artifact convention`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-285/define-baseline-vs-candidate-audio-qa-artifact-convention`
- Project: `P000 | Repo Ops / QA / Workflow`
- Milestone: `P000 | Repo Ops / QA / Workflow`
- Status: `Done`
- Created: `2026-04-26`
- Started: `2026-04-26`
- Finished: `2026-04-26`
- Deleted from Linear: `2026-04-26`
- Branch: `feature/riotbox-285-define-baseline-vs-candidate-audio-qa-artifact-convention`
- Linear branch: `feature/riotbox-285-define-baseline-vs-candidate-audio-qa-artifact-convention`
- PR: `#275`
- Merge commit: `6792352`
- Labels: `workflow`, `benchmark`
- Follow-ups: `RIOTBOX-286`

## Why This Ticket Existed

`RIOTBOX-284` created the first W-30 preview smoke listening-pack convention, but it still only described a candidate render. Audio QA needs a stable baseline-vs-candidate artifact shape before later comparison tooling can be added without path drift.

## What Shipped

- Added a durable audio QA artifact convention for baseline, candidate, metrics, and notes files.
- Updated the W-30 preview smoke pack to use the baseline-vs-candidate shape.
- Linked the convention from benchmark indexes and the active audio QA workflow spec.

## Verification

- `git diff --check`
- `just ci`
- GitHub Actions `rust-ci`

## Notes

- Docs/ops convention only; no comparison engine, generated pack runner, or CI audio artifact gate changed.
- The fast-forward merge means the feature commit is also the merge commit on `main`.
