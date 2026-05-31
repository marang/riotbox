# `RIOTBOX-1080` Tighten export QA evidence identity validation before stem scopes

- Ticket: `RIOTBOX-1080`
- Title: `Tighten export QA evidence identity validation before stem scopes`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1080/tighten-export-qa-evidence-identity-validation-before-stem-scopes`
- Project: `P016 | Pro Workflow / Export`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-31`
- Started: `2026-05-31`
- Finished: `2026-05-31`
- Branch: `feature/riotbox-1080-export-qa-evidence-identity`
- Linear branch: `feature/riotbox-1080-tighten-export-qa-evidence-identity-validation-before-stem`
- Assignee: `Markus`
- Labels: None
- PR: `#1056 (https://github.com/marang/riotbox/pull/1056)`
- Merge commit: `498691eb4e7ad27abe8da80a75f39d18e9b9038e`
- Deleted from Linear: `2026-05-31`
- Verification: `cargo test -p riotbox-core export_qa -- --nocapture`; `git diff --check`; `just ci`; `GitHub rust-ci passed on PR #1056`
- Docs touched: `docs/specs/audio_qa_workflow_spec.md`
- Follow-ups: `None`

## Why This Ticket Existed

P016 export QA evidence policies must reject placeholder identity before real stem scopes depend on them.

## What Shipped

- Added invalid evidence failures for lineage and fallback comparison evidence.
- Rejected blank source graph ids/hashes and blank capture ids when lineage evidence is required.
- Rejected blank fallback reference identities and metricless fallback comparison evidence when fallback comparison evidence is required.
- Split evidence-policy tests into a semantic module under the Rust review budget.

## Notes

- No stem writing, DAW export, live recording export, real source-vs-fallback rendering, or threshold interpretation shipped.
