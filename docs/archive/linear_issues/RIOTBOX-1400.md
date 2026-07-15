# `RIOTBOX-1400` P023: Promote dense-break performance policy into the shared live runtime

- Ticket: `RIOTBOX-1400`
- Title: `P023: Promote dense-break performance policy into the shared live runtime`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1400/p023-promote-dense-break-performance-policy-into-the-shared-live`
- Project: `P023 | Sound Excellence / Production Quality`
- Milestone: `M2 | Live Dense-Break Golden Path`
- Status: `Done`
- Created: `2026-07-11`
- Started: `2026-07-12`
- Finished: `2026-07-15`
- Branch: `feature/riotbox-1400-p023-promote-dense-break-performance-policy-into-the-shared`
- Linear branch: `feature/riotbox-1400-p023-promote-dense-break-performance-policy-into-the-shared`
- Assignee: `Markus`
- Labels: `Audio`, `Core`, `Feature`, `Feral`
- PR: `#1366 (https://github.com/marang/riotbox/pull/1366)`
- Merge commit: `dc297710e661f7bd891f99a60ed4b85419e5854f`
- Deleted from Linear: `2026-07-15`
- Verification: `Full just ci, exact dense-break live-path smoke, targeted core/app/audio regressions, branch code/Rust review, and GitHub Actions Rust CI passed.`
- Docs touched: `AGENTS.md; docs/phase_definition_of_done.md; docs/research_decision_log.md; docs/specs/audio_core_spec.md; docs/specs/audio_qa_workflow_spec.md; docs/specs/replay_model_spec.md; docs/specs/session_file_spec.md`
- Follow-ups: `RIOTBOX-1401: source-derived eight-bar hook, pressure lift, destructive contrast, and changed-return musical-alpha slice.`

## Why This Ticket Existed

Promote dense-break performance ownership into the exact shared live runtime without overstating rejected musical quality.

## What Shipped

- Added typed source-backed live performance intent and explicit bass ownership across W-30, TR-909, and MC-202.
- Added fail-closed timing, fallback, missing-plan, and legacy-plan behavior on the existing Session/Source Graph spine.
- Added reproducible exact callback-path audio QA and preserved the human-rejected result as a bounded contract enabler for RIOTBOX-1401.

## Notes

- Human listening rejected the final RIOTBOX-1400 all-lane candidate as too repetitive and less useful than the source; no musical-alpha pass was claimed.
