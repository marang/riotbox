# P012 Jam / Source Timing Surface Review - 2026-05-21

Scope: cadence review after RIOTBOX-869 through RIOTBOX-874, focused on the
Source Timing summary boundary and Jam / Source musician-facing surfaces.

## Summary

The current direction is sound: Source Graph remains the durable timing truth,
while `SourceTimingSummaryView` now owns cue, actionability, compact beat /
downbeat / phrase status, anchor evidence, and observer-facing readiness fields.
This reduces drift across Jam, Source, Help, observer snapshots, and QA readouts.

No blocking architecture issue was found. One minor product-surface gap remains:
the newest actionability phrase is available in the shared summary, Source panel,
Help, and observer snapshots, but the primary compact Jam Trust line still does
not show it.

## Findings

- **Location**: `crates/riotbox-app/src/ui/jam_perform_layout.rs:60`,
  `crates/riotbox-app/src/ui/source_trust_summary.rs:190`,
  `docs/specs/tui_screen_spec.md:83`
- **Category**: scope
- **Severity**: minor
- **Title**: Jam Trust readiness line omits shared Source Timing actionability
- **Description**: The TUI spec says compact Source Timing readiness in Jam
  perform / inspect views includes the shared actionability phrase, and the
  shared summary now owns that phrase. The Trust panel still renders
  `source_timing_readiness_line(...)`, which shows cue, grid use, phase, quality,
  and anchor type, but not the actionability text. Help / Start Here and Source
  do show the phrase, so this is a surface consistency gap rather than a broken
  control path.
- **Suggestion**: Add a bounded Jam perform / inspect cue that includes
  `timing.actionability`, or intentionally narrow the spec if the Trust panel is
  meant to stay ultra-compact.

## Dimensions

- Architecture / boundaries: Source Graph is still the durable timing truth;
  presentation stays in `SourceTimingSummaryView`.
- Design consistency: good overall; one Jam line remains less explicit than
  Source / Help.
- Maintainability: field ownership is now clearer than screen-local policy
  matching.
- Coupling: no new lane-local timing system or app-local replay truth.
- Safety / performance: no realtime audio path or blocking I/O impact.
- Recommendation: next bounded slice should close the Jam compact readiness gap
  before moving actionability into deeper all-lane automation.
