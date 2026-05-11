# `RIOTBOX-781` Prove strict auto-grid and fallback Source Timing pack paths stay distinct

- Ticket: `RIOTBOX-781`
- Title: `Prove strict auto-grid and fallback Source Timing pack paths stay distinct`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-781/prove-strict-auto-grid-and-fallback-source-timing-pack-paths-stay`
- Project: `P012 | Source Timing Intelligence`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-11`
- Started: `2026-05-11`
- Finished: `2026-05-11`
- Branch: `feature/riotbox-781-prove-strict-auto-grid-and-fallback-source-timing-pack-paths`
- Linear branch: `feature/riotbox-781-prove-strict-auto-grid-and-fallback-source-timing-pack-paths`
- Assignee: `Markus`
- Labels: `benchmark`, `timing`
- PR: `#775 (https://github.com/marang/riotbox/pull/775)`
- Merge commit: `a207d294786ef185cd6a741dab602ab4efda4bb3`
- Deleted from Linear: `2026-05-11`
- Verification: `bash -n scripts/correlate_generated_feral_grid_observer.sh`; `git diff --check`; `just observer-audio-correlate-generated-feral-grid`; `just ci`
- Docs touched: `docs/specs/source_timing_intelligence_spec.md`
- Follow-ups: `None`

## Why This Ticket Existed

Generated Feral-grid QA needed explicit proof that strict auto-grid and fallback/manual-confirm Source Timing paths stay distinct. The slice protects against weak timing evidence silently being treated as locked timing in observer/audio summaries.

## What Shipped

- Tightened generated Feral-grid observer QA to assert control, manifest, output, and alignment grid-use fields for cautious/manual-confirm, user override, fallback, and locked-grid paths.
- Documented the strict Source Timing grid-use proof requirement in the Source Timing Intelligence spec.

## Notes

- No detector threshold or production arbitrary-audio beat-detection behavior changed.
