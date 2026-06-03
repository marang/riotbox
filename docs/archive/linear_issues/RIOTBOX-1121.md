# `RIOTBOX-1121` P016: Add stem-package writer planning contract

- Ticket: `RIOTBOX-1121`
- Title: `P016: Add stem-package writer planning contract`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1121/p016-add-stem-package-writer-planning-contract`
- Project: `P016 | Pro Workflow / Export`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-03`
- Started: `2026-06-03`
- Finished: `2026-06-03`
- Branch: `feature/riotbox-1121-p016-add-stem-package-writer-planning-contract`
- Linear branch: `feature/riotbox-1121-p016-add-stem-package-writer-planning-contract`
- Assignee: `Markus`
- Labels: `Docs`, `workflow`
- PR: `#1100 (https://github.com/marang/riotbox/pull/1100)`
- Merge commit: `1195f806615f68ec7dec0583ded1c2e43bc55658`
- Deleted from Linear: `2026-06-03`
- Verification: `scripts/run_compact.sh /tmp/riotbox-1121-just-ci.log just ci; GitHub rust-ci passed`
- Docs touched: `docs/specs/action_lexicon_spec.md; docs/specs/session_file_spec.md; docs/specs/audio_qa_workflow_spec.md`
- Follow-ups: `RIOTBOX-1122`

## Why This Ticket Existed

P016 needed a concrete stem-package writer planning contract before implementation so future package writing preserves realtime isolation, replay/restore boundaries, explicit QA gate order, and Session receipt truth.

## What Shipped

- Documented reusable product-export pieces, new stem-package writer pieces, writer gate order, realtime/replay boundaries, and the manifest/proof non-circular identity precondition; created RIOTBOX-1122 for the self-hash follow-up.

## Notes

- No runnable export action or package writer shipped; no audible behavior changed.
