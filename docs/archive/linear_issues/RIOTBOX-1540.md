# `RIOTBOX-1540` Reject physical aliases between W-30 WAV and metrics outputs

- Ticket: `RIOTBOX-1540`
- Title: `Reject physical aliases between W-30 WAV and metrics outputs`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1540/reject-physical-aliases-between-w-30-wav-and-metrics-outputs`
- Project: `P000 | Repo Ops / QA / Workflow`
- Milestone: `None`
- Status: `Done`
- Created: `2026-10-01`
- Started: `2026-10-01`
- Finished: `2026-10-01`
- Branch: `feature/riotbox-1540-w30-output-alias-integrity`
- Linear branch: `feature/riotbox-1540-reject-physical-aliases-between-w-30-wav-and-metrics-outputs`
- Assignee: `Markus`
- Labels: `Audio`, `review-followup`
- PR: `#1624 (https://github.com/marang/riotbox/pull/1624)`
- Merge commit: `57fa0184c06257a93434454614b72a995a4c9a5b`
- Deleted from Linear: `2026-10-01`
- Verification: `133 focused Debug/Release tests; 141 CLI records and 85 artifact hashes unchanged; source-free just ci zero; native run 36841422040 both required checks SUCCESS.`
- Docs touched: `None`
- Follow-ups: `RIOTBOX-1541 independently confirms and addresses Feral pack output-role coupling.`

## Why This Ticket Existed

W-30 renderer accepted physically coupled WAV and metrics outputs, allowing a successful command to replace RIFF audio with Markdown.

## What Shipped

- Reject mutual output aliases in explicit-source and explicit synthetic QA modes before rendering/publication; retain one shared regular-file admission and identity backend.

## Notes

- Merge 57fa0184c06257a93434454614b72a995a4c9a5b; no real sources, Holdout, commercial audio, DAW/playback or subagents. Stable namespace only.
