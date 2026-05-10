# `RIOTBOX-274` Document Capture source-readiness handoff cue

- Ticket: `RIOTBOX-274`
- Title: `Document Capture source-readiness handoff cue`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-274/document-capture-source-readiness-handoff-cue`
- Project: `P007 | W-30 MVP`
- Milestone: `P007 | W-30 MVP`
- Status: `Done`
- Created: `2026-04-26`
- Started: `2026-04-26`
- Finished: `2026-04-26`
- Deleted from Linear: `2026-04-26`
- Branch: `feature/riotbox-274-document-capture-source-readiness-handoff-cue`
- Linear branch: `feature/riotbox-274-document-capture-source-readiness-handoff-cue`
- PR: `#264`
- Merge commit: `d3c7a5c`
- Labels: `review-followup`, `ux`
- Follow-ups: `RIOTBOX-275`

## Why This Ticket Existed

`RIOTBOX-273` added compact `src` / `fallback` readiness cues to Capture handoff copy. The TUI spec needed to preserve that behavior as part of the Capture screen contract so future UI cleanup does not accidentally remove the handoff readiness cue.

## What Shipped

- Documented that Capture handoff lines may show compact `src` / `fallback` readiness when suggesting `[w] hit` or `[p]->[w]`.
- Added an MVP rule that the readiness cue must stay visible without outranking the primary gesture.

## Verification

- `git diff --check`
- `just ci`
- GitHub Actions `rust-ci`

## Notes

- Docs/spec-only slice; no runtime behavior, audio behavior, or Capture layout implementation changed.
- Feature commit: `d78bfce`.
