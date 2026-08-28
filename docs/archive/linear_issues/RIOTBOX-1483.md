# `RIOTBOX-1483` Deliver the qualified W-30 hook as a musician-ready export handoff

- Ticket: `RIOTBOX-1483`
- Title: `Deliver the qualified W-30 hook as a musician-ready export handoff`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1483/deliver-the-qualified-w-30-hook-as-a-musician-ready-export-handoff`
- Project: `P016 | Pro Workflow / Export`
- Milestone: `None`
- Status: `Done`
- Created: `2026-08-28`
- Started: `2026-08-28`
- Finished: `2026-08-28`
- Branch: `feature/riotbox-1483-w30-hook-musician-handoff`
- Linear branch: `feature/riotbox-1483-deliver-the-qualified-w-30-hook-as-a-musician-ready-export`
- Assignee: `Markus`
- Labels: `Audio`, `Core`, `Feature`
- PR: `#1500 (https://github.com/marang/riotbox/pull/1500)`
- Merge commit: `916990009bd0e7fd277cf330071b23a2f9edd72c`
- Deleted from Linear: `2026-08-28`
- Verification: `focused W-30 ready and blocked regression tests passed`; `just ci passed`; `GitHub rust-ci passed`
- Docs touched: `README.md`, `docs/execution_roadmap.md`, `docs/research_decision_log.md`, `docs/specs/action_lexicon_spec.md`, `docs/specs/session_file_spec.md`
- Follow-ups: `Next bounded P016 workflow from RIOTBOX-1036; TUI/Ghost, DAW-project creation, live recording, Holdout, and release remain separate.`

## Why This Ticket Existed

The qualified RIOTBOX-1482 W-30 hook had a file-producing operator path but no direct, understandable handoff that told a musician which file to use and how its tempo and loop geometry relate to the Session.

## What Shipped

- Added one safely quoted just w30-hook-handoff command over the existing stem_package.w30_hook_loop_v4 action and atomic writer.
- Added a deterministic non-persisted musician summary with exact WAV, manifest, proof, Session BPM, two-bar geometry, audio format, and source/capture lineage.
- Preserved the accepted V4 audio, manifest, proof, receipt, replay, and action contracts while keeping invalid sessions fail-closed without a final package.

## Notes

- Branch review found and fixed unsafe shell interpolation in the new wrapper; no remaining code-review or Rust-review findings.
- No audio bytes, source access, Holdout access, commercial reference, playback, or listening verdict changed.
