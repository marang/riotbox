# `RIOTBOX-980` Evaluate ratatui canvas waveform fallback for Source Map

- Ticket: `RIOTBOX-980`
- Title: `Evaluate ratatui canvas waveform fallback for Source Map`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-980/evaluate-ratatui-canvas-waveform-fallback-for-source-map`
- Project: `P012 | Source Timing Intelligence`
- Milestone: `None`
- Status: `Done`
- Created: `2026-05-23`
- Started: `2026-05-23`
- Finished: `2026-05-26`
- Branch: `feature/riotbox-980-evaluate-ratatui-canvas-waveform-fallback-for-source-map`
- Linear branch: `feature/riotbox-980-evaluate-ratatui-canvas-waveform-fallback-for-source-map`
- Assignee: `Markus`
- Labels: `Improvement`, `timing`, `ux`
- PR: `#972 (https://github.com/marang/riotbox/pull/972)`
- Merge commit: `c52078e501303ce0b91ceba52c180f64dc059190`
- Deleted from Linear: `2026-05-26`
- Verification: `git diff --check`; `manual review of updated specs and spike artifact`
- Docs touched: `docs/plans/source_transport_map_capture_plan.md`, `docs/research_decision_log.md`, `docs/reviews/source_map_waveform_canvas_spike_2026-05-23.md`, `docs/specs/source_timing_intelligence_spec.md`, `docs/specs/tui_screen_spec.md`
- Follow-ups: `None`

## Why This Ticket Existed

Evaluate whether Ratatui Canvas should become the default Source Map waveform renderer or remain an optional expanded view without creating a second timing/capture authority.

## What Shipped

- Added the Source Map waveform Canvas spike review comparing block rows, line rows, and Canvas/Braille options.
- Documented default Source Map rendering as one/two-line block rows with explicit marker/text rows.
- Kept Ratatui Canvas/Braille as a future expanded Source/Lab option consuming the same SourceMapView projection contract.

## Notes

- Docs-only slice; no runtime or audio output behavior changed.
