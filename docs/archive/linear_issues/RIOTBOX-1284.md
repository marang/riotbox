# `RIOTBOX-1284` Persist or reconstruct MC-202 source phrase plans through replay and restore

- Ticket: `RIOTBOX-1284`
- Title: `Persist or reconstruct MC-202 source phrase plans through replay and restore`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1284/persist-or-reconstruct-mc-202-source-phrase-plans-through-replay-and`
- Project: `P023 | Sound Excellence / Production Quality`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-15`
- Started: `2026-06-18`
- Finished: `2026-06-18`
- Branch: `feature/riotbox-1284-mc202-phrase-plan-replay`
- Linear branch: `feature/riotbox-1284-persist-or-reconstruct-mc-202-source-phrase-plans-through`
- Assignee: `Markus`
- Labels: `Audio`
- PR: `#1257 (https://github.com/marang/riotbox/pull/1257)`
- Merge commit: `09c85045a15a9c486cc5f7f967e8d0920a09c38f`
- Deleted from Linear: `2026-06-18`
- Verification: `cargo test -p riotbox-app mc202_ -- --nocapture; cargo test -p riotbox-core replay:: -- --nocapture; just audio-qa-ci; just ci`
- Docs touched: `docs/specs/session_file_spec.md; docs/specs/replay_model_spec.md`
- Follow-ups: `None`

## Why This Ticket Existed

MC-202 source-derived phrases must survive action-log and snapshot replay without falling back to hardcoded audio.

## What Shipped

- Persisted MC-202 source phrase decisions on ActionCommitRecord, restored trusted persisted plans through JamAppState replay/from-parts, kept old/no-evidence logs degraded or silent, and documented the Session/replay contract.

## Notes

- None
