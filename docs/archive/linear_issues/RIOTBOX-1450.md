# `RIOTBOX-1450` P023: Fail closed on mismatched MC-202 performer intent

- Ticket: `RIOTBOX-1450`
- Title: `P023: Fail closed on mismatched MC-202 performer intent`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1450/p023-fail-closed-on-mismatched-mc-202-performer-intent`
- Project: `P023 | Sound Excellence / Production Quality`
- Milestone: `M3 | Human-Passed Musical Alpha`
- Status: `Done`
- Created: `2026-08-21`
- Started: `2026-08-21`
- Finished: `2026-08-21`
- Branch: `feature/riotbox-1450-p023-fail-closed-on-mismatched-mc-202-performer-intent`
- Linear branch: `feature/riotbox-1450-p023-fail-closed-on-mismatched-mc-202-performer-intent`
- Assignee: `Markus`
- Labels: `Audio`, `Bug`, `Core`, `review-followup`
- PR: `#1420 (https://github.com/marang/riotbox/pull/1420)`
- Merge commit: `233c6d16`
- Deleted from Linear: `2026-08-21`
- Verification: `just ci: pass`; `GitHub Rust CI: pass`
- Docs touched: `docs/research_decision_log.md`, `docs/specs/session_file_spec.md`, `docs/specs/action_lexicon_spec.md`, `docs/specs/audio_core_spec.md`, `docs/specs/replay_model_spec.md`
- Follow-ups: `Future MC-202 audible exploration remains separate; RIOTBOX-1450 makes no musical-quality claim.`

## Why This Ticket Existed

Prevent explicit MC-202 performer intent from silently selecting and rendering a different source-derived candidate family.

## What Shipped

- Added typed role-to-family compatibility and fail-closed candidate rejection before MC-202 selection.
- Persisted visible degraded silence when no compatible family survives, including silent restore/replay of historical mismatches.
- Corrected the dense-break diagnostic and stale fixtures to use compatible role/family ownership.

## Notes

- None
