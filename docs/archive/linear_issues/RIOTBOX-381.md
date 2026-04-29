# `RIOTBOX-381` Add validator fixture tests for observer/audio JSON summaries

- Ticket: `RIOTBOX-381`
- Title: `Add validator fixture tests for observer/audio JSON summaries`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-381/add-validator-fixture-tests-for-observeraudio-json-summaries`
- Project: `P000 | Repo Ops / QA / Workflow`
- Milestone: `Repo Ops / QA / Workflow`
- Status: `Done`
- Created: `2026-04-29`
- Started: `2026-04-29`
- Finished: `2026-04-29`
- Branch: `feature/riotbox-381-observer-audio-validator-fixtures`
- Linear branch: `feature/riotbox-381-add-validator-fixture-tests-for-observeraudio-json-summaries`
- Assignee: `Markus`
- Labels: `benchmark`, `workflow`, `Audio`
- PR: `#370`
- Merge commit: `f4dc0235e31bee2db90adfe95207e23acb5faefb`
- Deleted from Linear: `Not deleted yet`
- Verification: `just observer-audio-summary-validator-fixtures`, `just audio-qa-ci`, `cargo fmt --check`, `git diff --check`, `just ci`, Rust file line budget check
- Docs touched: `docs/benchmarks/observer_audio_summary_json_contract_2026-04-29.md`, `docs/specs/audio_qa_workflow_spec.md`
- Follow-ups: `None`

## Why This Ticket Existed

The observer/audio summary validator needed its own fixture coverage so it would accept valid failure summaries and reject malformed summaries instead of relying only on the happy-path fixture.

## What Shipped

- added a valid failure-summary JSON fixture with `null` output metrics
- added an invalid schema-marker JSON fixture that must be rejected
- added `just observer-audio-summary-validator-fixtures`
- wired validator fixture checks into `just audio-qa-ci` and GitHub Actions
- documented the validator fixture coverage

## Notes

- this was QA hardening only; no audio behavior changed
