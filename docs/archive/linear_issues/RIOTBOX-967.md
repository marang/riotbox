# `RIOTBOX-967` Source transport, adaptive source map, monitor, and capture workflow

- Ticket: `RIOTBOX-967`
- Title: `Source transport, adaptive source map, monitor, and capture workflow`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-967/source-transport-adaptive-source-map-monitor-and-capture-workflow`
- Project: `P012 | Source Timing Intelligence`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-23`
- Started: `2026-05-23`
- Finished: `2026-05-26`
- Branch: `feature/riotbox-967-source-transport-adaptive-source-map-monitor-and-capture`
- Linear branch: `feature/riotbox-967-source-transport-adaptive-source-map-monitor-and-capture`
- Assignee: `Unassigned`
- Labels: `Feature`, `timing`, `ux`
- PR: None
- Merge commit: `None`
- Deleted from Linear: `2026-05-26`
- Verification: `child PR stack RIOTBOX-968 through RIOTBOX-992 merged`; `cargo fmt --check / focused cargo tests / git diff --check / just ci across child closeouts`; `just source-transport-map-capture-probe on RIOTBOX-989 e2e QA slice`
- Docs touched: `docs/plans/source_transport_map_capture_plan.md`, `docs/research_decision_log.md`
- Follow-ups: `None`

## Why This Ticket Existed

Track and close the full Source transport, adaptive Source Map, monitor, and capture workflow as one musician-facing parent plan.

## What Shipped

- Completed source monitor mode contracts and realtime source/blend/riotbox routing.
- Completed adaptive block Source Map projection, timing-grid confirmation/revert, and confirmation-aware trust/readiness surfaces.
- Completed source navigation by bar/phrase, capture length intent, capture preview/observer surfaces, committed source-window semantics, and Capture screen labels.
- Completed restore/replay fidelity, audible seek-output proof, CI-safe e2e workflow probe, and decoded-WAV Source Map bucket ingest proof.

## Notes

- Parent ticket had no direct PR; implementation shipped through child tickets RIOTBOX-968 through RIOTBOX-992.
