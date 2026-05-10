# `RIOTBOX-280` Document source-backed W-30 hit QA coverage

- Ticket: `RIOTBOX-280`
- Title: `Document source-backed W-30 hit QA coverage`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-280/document-source-backed-w-30-hit-qa-coverage`
- Project: `P007 | W-30 MVP`
- Milestone: `P007 | W-30 MVP`
- Status: `Done`
- Created: `2026-04-26`
- Started: `2026-04-26`
- Finished: `2026-04-26`
- Deleted from Linear: `2026-04-26`
- Branch: `feature/riotbox-280-document-source-backed-w-30-hit-regression-in-audio-qa-notes`
- Linear branch: `feature/riotbox-280-document-source-backed-w-30-hit-regression-in-audio-qa-notes`
- PR: `#270`
- Merge commit: `a7fbbee`
- Labels: `review-followup`, `benchmark`
- Follow-ups: `RIOTBOX-281`

## Why This Ticket Existed

`RIOTBOX-279` added source-backed W-30 hit regression coverage. The audio QA workflow spec needed to reflect that this seam is now covered by focused app/runtime regression tests while fuller listening-pack gates remain future work.

## What Shipped

- Updated `docs/specs/audio_qa_workflow_spec.md` to list source-backed W-30 `[w] hit` reuse coverage in the current operational repo status.

## Verification

- `git diff --check`
- `just ci`
- GitHub Actions `rust-ci`

## Notes

- Docs-only QA workflow note; no audio runtime, fixture harness, or playback behavior changed.
- This landed as a merge commit because `main` had already received the RIOTBOX-279 archive commit after the stacked feature commit.
