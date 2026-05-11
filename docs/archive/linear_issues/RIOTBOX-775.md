# `RIOTBOX-775` Reject observer/audio grid-use policy contradictions

- Ticket: `RIOTBOX-775`
- Title: `Reject observer/audio grid-use policy contradictions`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-775/reject-observeraudio-grid-use-policy-contradictions`
- Project: `P012 | Source Timing Intelligence`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-11`
- Started: `2026-05-11`
- Finished: `2026-05-11`
- Branch: `feature/riotbox-775-reject-observeraudio-grid-use-policy-contradictions`
- Linear branch: `feature/riotbox-775-reject-observeraudio-grid-use-policy-contradictions`
- Assignee: `Markus`
- Labels: `benchmark`, `timing`
- PR: `#769 (https://github.com/marang/riotbox/pull/769)`
- Merge commit: `cbfe0cc2122d8fcb0af7db511d64eaf11651537f`
- Deleted from Linear: `2026-05-11`
- Verification: `cargo fmt --check; cargo test -p riotbox-app --bin observer_audio_correlate; just observer-audio-correlate-generated-feral-grid; git diff --check; just ci`
- Docs touched: `docs/benchmarks/observer_audio_summary_json_contract_2026-04-29.md; docs/specs/source_timing_intelligence_spec.md`
- Follow-ups: `None.`

## Why This Ticket Existed

Strict observer/audio QA needed to fail contradictory grid_use / grid BPM decision evidence instead of merely displaying it.

## What Shipped

- Added a grid_use policy check to observer/audio output evidence failures, preserved user override tolerance, added negative Rust tests, and documented the strict correlation behavior.

## Notes

- None
