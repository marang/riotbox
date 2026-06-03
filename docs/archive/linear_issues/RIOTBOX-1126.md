# `RIOTBOX-1126` P016: Specify first bounded stem-package writer boundary

- Ticket: `RIOTBOX-1126`
- Title: `P016: Specify first bounded stem-package writer boundary`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1126/p016-specify-first-bounded-stem-package-writer-boundary`
- Project: `P016 | Pro Workflow / Export`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-03`
- Started: `2026-06-03`
- Finished: `2026-06-03`
- Branch: `feature/riotbox-1126-p016-specify-first-bounded-stem-package-writer-boundary`
- Linear branch: `feature/riotbox-1126-p016-specify-first-bounded-stem-package-writer-boundary`
- Assignee: `Markus`
- Labels: `workflow`
- PR: `#1105 (https://github.com/marang/riotbox/pull/1105)`
- Merge commit: `0cde1aecdcbf9a50a37269f6cd100a486ced9a4a`
- Deleted from Linear: `2026-06-03`
- Verification: `git diff --check; scripts/run_compact.sh /tmp/riotbox-1126-just-ci.log just ci; GitHub rust-ci passed`
- Docs touched: `docs/specs/action_lexicon_spec.md; docs/specs/session_file_spec.md; docs/specs/audio_qa_workflow_spec.md; docs/research_decision_log.md`
- Follow-ups: `Create the next P016 implementation slice for the first CI-safe writer skeleton under stem_package.local_ci_package_v1.`

## Why This Ticket Existed

P016 needed the first bounded stem-package writer boundary before any code could write packages or surface export.stem_package as runnable.

## What Shipped

- Specified stem_package.local_ci_package_v1, deterministic offline stem source requirements, final package layout, exact commit/receipt order, unsupported-scope rule, reusable/new writer pieces, minimal output proof, and RBX-064 decision-log entry.

## Notes

- Spec-only slice. No package writer runs, no files are emitted, and no audible behavior changes.
