# Source Map Waveform Rendering Spike

Date: 2026-05-23
Ticket: RIOTBOX-980
Phase: P012 Source Timing Intelligence

## Question

Should Source Map waveform rendering move from the current compact block rows to
`ratatui::widgets::canvas::Canvas`, or should Canvas stay optional for a later
expanded Source/Lab view?

## Current Contract

The current Source Map model is a compact product surface:

- `energy`: one line of bucketed loudness / energy characters
- `peaks`: one line of transient / anchor emphasis
- `bars`: bar markers when timing readiness allows bar-grid use
- `play`: current transport position from Session transport state
- text labels carry mode, trust, navigation, and capture safety

This is intentionally not a DAW editor. It answers where the musician is, whether
the grid can be trusted, and whether capture will be bar-accurate or fallback.

## Renderer Comparison

Examples below use the same conceptual 32-column source map.

### One-line block rows

```text
energy ▁▂▅▇██▇▅▂▁▂▅▇██▇▅▂▁▁▂▅▇█▇▅▂▁▂▅▇█
peaks  ..█...▇.....█..▇...█....▇...█...
bars   |...|...|...|...|...|...|...|...
play               ^
```

Strengths:

- readable in narrow terminal panels
- stable in monochrome snapshots
- cheap to render as plain `Line` / `Paragraph`
- easy to test as strings in core and TUI snapshots
- aligns with the current two-row musician contract

Weaknesses:

- coarse amplitude shape
- not suitable for inspecting fine waveform contours

### Two-line block rows

```text
energy ▁▂▅▇██▇▅▂▁▂▅▇██▇▅▂▁▁▂▅▇█▇▅▂▁▂▅▇█
       ▁▁▂▅▇█▇▅▂▁▁▂▅▇█▇▅▂▁▂▅▇██▇▅▂▁▁▂▅
peaks  ..█...▇.....█..▇...█....▇...█...
bars   |...|...|...|...|...|...|...|...
play               ^
```

Strengths:

- best near-term compromise for source orientation
- gives the musician more shape without a separate widget model
- remains plain text and snapshot-friendly

Weaknesses:

- consumes one additional Source panel row
- still summarizes buckets rather than drawing sample-accurate waveform detail

### Simple line rows

```text
wave   _/^^\__/^^^\___/^^\____/^\__/^^^\
peaks  ..|...|.....|..|...|....|...|...
bars   |---|---|---|---|---|---|---|---
play               ^
```

Strengths:

- familiar visual language
- can show contour with fewer heavy glyphs

Weaknesses:

- diagonal line continuity is fragile in terminal fonts
- hard to combine with bars, peaks, playhead, and capture markers without
  becoming visually noisy
- worse for deterministic bucket testing than block rows

### Ratatui Canvas

`ratatui` 0.29 exposes `Canvas` with explicit `x_bounds`, `y_bounds`, a `paint`
closure, and marker choices including `Dot`, `Block`, `Bar`, `Braille`, and
`HalfBlock`. Current upstream docs also note marker behavior changes around
`Block` / `Bar`, so version-specific marker choices must be treated carefully.

Example shape:

```text
canvas Marker::Braille, x_bounds [0.0, 32.0], y_bounds [-1.0, 1.0]

⡀⣀⣤⣶⣿⣿⣶⣤⣀⡀⢀⣤⣶⣿⣷⣦⡀⣀⣤⣶⣿⣶⣤⣀
```

Strengths:

- highest density available inside a terminal
- useful for an expanded Source/Lab waveform view
- can draw overlays such as lines or rectangles when enough vertical space is
  available

Weaknesses:

- Braille is dense but harder to parse at a glance than blocks
- terminal/font support affects output, especially for Braille
- a useful Canvas needs height; one or two rows do not justify the widget
- snapshot tests become more sensitive to Ratatui marker/version behavior
- it must still render separate textual trust, playhead, bar, and capture cues

## Decision

Keep the default Source Map renderer as one or two plain-text block rows.

Use Canvas only as a later optional expanded Source/Lab renderer when all of
these are true:

- the Source Map bucket contract is stable
- the view has enough height to render more than two waveform rows
- textual mode, trust, playhead, bar, and capture cues remain visible
- monochrome snapshots prove the same state without relying on color
- Canvas consumes the same `SourceMapView` / source-window projection data rather
  than introducing a second Source Map truth

## Implementation Guidance

Near-term Source Map rendering should improve the block contract first:

- keep `energy` and `peaks` as plain rows
- allow an optional second energy/detail row when the Source panel has room
- keep `bars`, `play`, and later capture range as separate marker rows
- avoid Braille as the default musician-facing row
- do not use Canvas to bypass Session transport, timing readiness, or capture
  length contracts

Canvas remains useful, but as an expanded view, not the default map.
