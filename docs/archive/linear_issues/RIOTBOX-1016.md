# `RIOTBOX-1016` P012: Align audio QA spec with MC-202 Feral-grid proof output

- Ticket: `RIOTBOX-1016`
- Title: `P012: Align audio QA spec with MC-202 Feral-grid proof output`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1016/p012-align-audio-qa-spec-with-mc-202-feral-grid-proof-output`
- Project: `None`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-27`
- Started: `2026-05-27`
- Finished: `2026-05-27`
- Branch: `feature/riotbox-1016-p012-align-audio-qa-spec-with-mc-202-feral-grid-proof-output`
- Linear branch: `feature/riotbox-1016-p012-align-audio-qa-spec-with-mc-202-feral-grid-proof-output`
- Assignee: `Markus`
- Labels: `Audio`, `Docs`, `timing`
- PR: `#999 (https://github.com/marang/riotbox/pull/999)`
- Merge commit: `52bd337e5f9e0a13b02fa5f2e9df156c2cb8de96`
- Deleted from Linear: `2026-05-27`
- Verification: `Focused Feral-grid MC-202 docs drift search; git diff --check; PR #999 had no GitHub Actions runs published for docs-only head commit`
- Docs touched: `docs/specs/audio_qa_workflow_spec.md; docs/jam_recipes.md`
- Follow-ups: `None`

## Why This Ticket Existed

RIOTBOX-1015 changed MC-202 Feral-grid output from silent compatibility stem to primitive-renderer source-grid proof, but canonical docs still described the old silent contract.

## What Shipped

- Updated Audio QA workflow and Recipe 15 docs to describe primitive MC-202 proof output and mc202_source_grid_alignment as required lane evidence.

## Notes

- Remaining compatibility search hit is unrelated Session compatibility-label wording in docs/specs/session_file_spec.md.
