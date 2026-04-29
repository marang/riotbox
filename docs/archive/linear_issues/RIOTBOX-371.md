# `RIOTBOX-371` Reject collapsed metrics in strict observer/audio evidence

- Ticket: `RIOTBOX-371`
- Title: `Reject collapsed metrics in strict observer/audio evidence`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-371/reject-collapsed-metrics-in-strict-observeraudio-evidence`
- Project: `P000 | Repo Ops / QA / Workflow`
- Milestone: `Repo Ops / QA / Workflow`
- Status: `Done`
- Created: `2026-04-29`
- Started: `2026-04-29`
- Finished: `2026-04-29`
- Branch: `feature/riotbox-371-reject-collapsed-metrics-in-strict-observeraudio-evidence`
- Linear branch: `feature/riotbox-371-reject-collapsed-metrics-in-strict-observeraudio-evidence`
- Assignee: `Markus`
- Labels: `benchmark`, `workflow`, `Audio`
- PR: `#359`
- Merge commit: `a102cd54cc4b28c8e71cc472b7be253cc10ecf11`
- Deleted from Linear: `Not deleted yet`
- Verification: `cargo test -p riotbox-app --bin observer_audio_correlate`, `just ci`, `git diff --check`, Rust file line budget check
- Docs touched: `docs/specs/audio_qa_workflow_spec.md`
- Follow-ups: `RIOTBOX-372`

## Why This Ticket Existed

Strict observer/audio output evidence could not only check that metrics were present; zeroed metrics with `result: pass` still represented fallback-like or collapsed audio evidence.

## What Shipped

- strict output metrics must clear a small positive floor
- regression coverage rejects zeroed full-mix, low-band, and MC-202 delta metrics
- the audio QA workflow spec documents the non-collapsed metric rule

## Notes

- this was a QA correctness slice, not an audio rendering change
