# `RIOTBOX-1152` P016: Add DAW JSON package receipt evidence CLI apply path

- Ticket: `RIOTBOX-1152`
- Title: `P016: Add DAW JSON package receipt evidence CLI apply path`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1152/p016-add-daw-json-package-receipt-evidence-cli-apply-path`
- Project: `P016 | Pro Workflow / Export`
- Milestone: `None`
- Status: `Done`
- Created: `2026-06-03`
- Started: `2026-06-03`
- Finished: `2026-06-03`
- Branch: `feature/riotbox-1152-p016-add-daw-json-package-receipt-evidence-cli-apply-path`
- Linear branch: `feature/riotbox-1152-p016-add-daw-json-package-receipt-evidence-cli-apply-path`
- Assignee: `Markus`
- Labels: None
- PR: `#1131 (https://github.com/marang/riotbox/pull/1131)`
- Merge commit: `bbd4b544`
- Deleted from Linear: `2026-06-03`
- Verification: `cargo fmt; git diff --check; cargo test -p riotbox-app daw_session_json_package_evidence_apply -- --nocapture; just daw-session-json-package-evidence-apply-smoke; cargo test -p riotbox-app; scripts/run_compact.sh /tmp/riotbox-1152-just-ci.log just ci; GitHub rust-ci green`
- Docs touched: `docs/specs/action_lexicon_spec.md; docs/specs/audio_qa_workflow_spec.md; docs/specs/session_file_spec.md`
- Follow-ups: `Next P016 DAW export slice: derive observer lifecycle from Session DAW receipt evidence or add host/import proof boundary; do not make export.daw_session runnable yet.`

## Why This Ticket Existed

P016 needed the written DAW JSON package proof to become durable Session/Core receipt evidence instead of remaining only local package files.

## What Shipped

- Added --daw-session-json-package-evidence-apply, Session receipt evidence mutation for DAW JSON packages, a real-binary smoke target, focused parser/unit coverage, file-size-safe test split, and spec updates.

## Notes

- No package files, observer lifecycle events, DAW host/project files, host import proof, audible output proof, or musician-facing export.daw_session action were added.
