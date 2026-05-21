# `RIOTBOX-886` Expose generated Feral-grid observer/audio path details in P012 proof summary

- Ticket: `RIOTBOX-886`
- Title: `Expose generated Feral-grid observer/audio path details in P012 proof summary`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-886/expose-generated-feral-grid-observeraudio-path-details-in-p012-proof`
- Project: `P012 | Source Timing Intelligence`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-21`
- Started: `2026-05-21`
- Finished: `2026-05-21`
- Branch: `feature/riotbox-886-p012-generated-feral-grid-summary-details`
- Linear branch: `feature/riotbox-886-expose-generated-feral-grid-observeraudio-path-details-in`
- Assignee: `Markus`
- Labels: `timing`
- PR: `#880 (https://github.com/marang/riotbox/pull/880)`
- Merge commit: `9c7ff660c789c19e0457d41a17577b3626db61c0`
- Deleted from Linear: `2026-05-21`
- Verification: `python3 -m py_compile scripts/write_p012_all_lane_proof_summary.py scripts/validate_p012_all_lane_proof_summary.py; just p012-all-lane-proof-summary artifacts/audio_qa/local/p012_all_lane_source_grid_output_proof_summary.md; just p012-all-lane-source-grid-output-proof; just ci; git diff --check; GitHub Rust CI #2182 success`
- Docs touched: `None`
- Follow-ups: `RIOTBOX-887 centralizes app-side readiness actionability fallback`

## Why This Ticket Existed

The compact P012 proof summary collapsed generated Feral-grid observer/audio timing paths to a pass/fail component line even though the gate proved multiple grid-use and alignment paths.

## What Shipped

- Generated Feral-grid observer/audio summaries are persisted under artifacts/audio_qa/local/generated-feral-grid-observer-audio, and the compact P012 proof summary includes a table for cautious/manual, user-override, risky override, fallback, and locked-grid timing path outcomes.

## Notes

- None
