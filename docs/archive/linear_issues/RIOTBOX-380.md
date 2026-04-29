# `RIOTBOX-380` Add formal schema check for observer/audio JSON summaries

- Ticket: `RIOTBOX-380`
- Title: `Add formal schema check for observer/audio JSON summaries`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-380/add-formal-schema-check-for-observeraudio-json-summaries`
- Project: `P000 | Repo Ops / QA / Workflow`
- Milestone: `Repo Ops / QA / Workflow`
- Status: `Done`
- Created: `2026-04-29`
- Started: `2026-04-29`
- Finished: `2026-04-29`
- Branch: `feature/riotbox-380-observer-audio-json-schema-check`
- Linear branch: `feature/riotbox-380-add-formal-schema-check-for-observeraudio-json-summaries`
- Assignee: `Markus`
- Labels: `benchmark`, `workflow`, `Audio`
- PR: `#369`
- Merge commit: `ad2ad960ebedb90a5f086044d40ce3a5a9bd8123`
- Deleted from Linear: `Not deleted yet`
- Verification: `python3 -m py_compile scripts/validate_observer_audio_summary_json.py`, manual validator smoke with `null` metrics, `just observer-audio-correlate-json-fixture`, `just audio-qa-ci`, `cargo fmt --check`, `git diff --check`, `just ci`
- Docs touched: `docs/benchmarks/observer_audio_summary_json_contract_2026-04-29.md`, `docs/specs/audio_qa_workflow_spec.md`
- Follow-ups: `RIOTBOX-381`

## Why This Ticket Existed

The documented observer/audio JSON summary contract needed a mechanical validation path that did not require adding an external JSON Schema dependency.

## What Shipped

- added dependency-free `scripts/validate_observer_audio_summary_json.py`
- ran the validator from the committed JSON fixture smoke and GitHub Actions audio QA step
- allowed `null` metric values for valid failure summaries while the fixture gate still requires the passing happy path
- documented the validator in the JSON contract and audio QA workflow spec

## Notes

- this made the contract mechanically checkable without changing summary output shape
