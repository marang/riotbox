# `RIOTBOX-1538` Preserve W-30 comparison inputs before report and manifest publication

- Ticket: `RIOTBOX-1538`
- Title: `Preserve W-30 comparison inputs before report and manifest publication`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1538/preserve-w-30-comparison-inputs-before-report-and-manifest-publication`
- Project: `P000 | Repo Ops / QA / Workflow`
- Milestone: `None`
- Status: `Done`
- Created: `2026-10-01`
- Started: `2026-10-01`
- Finished: `2026-10-01`
- Branch: `feature/riotbox-1538-comparison-input-output-collision`
- Linear branch: `feature/riotbox-1538-preserve-w-30-comparison-inputs-before-report-and-manifest`
- Assignee: `Markus`
- Labels: `Audio`, `review-followup`
- PR: `#1620 (https://github.com/marang/riotbox/pull/1620)`
- Merge commit: `e7f874371661b58f0bcc9620c093db089520fdc9`
- Deleted from Linear: `2026-10-01`
- Verification: `117 matching passing Debug/Release test names; 47 exact normal CLI records and eight identical hashes; original input-loss loops reject without mutation; source-free just ci and exact-head native Rust/Windows-sidecar CI green`
- Docs touched: `RBX-408; audio QA owning spec; docs/reviews/riotbox_1538_comparison_input_preservation_2026-10-01.md`
- Follow-ups: `RIOTBOX-1539 integrated risk-directed architecture checkpoint; RIOTBOX-1509 remains separate pending causal Windows startup evidence`

## Why This Ticket Existed

Offline W-30 comparison must not destroy the metrics/WAV evidence it describes through report or manifest output

## What Shipped

- Shared physical-identity preflight over four actual referenced inputs and both writer-owned output paths; output self-alias and early missing/nonregular input rejection

## Notes

- Stable namespace only, no atomic pack/WAV content/Windows Audio/musical claim; no source corpus/device/DAW/playback/subagents; candidate code/contracts equal merged main with only four disjoint predecessor-archive differences
