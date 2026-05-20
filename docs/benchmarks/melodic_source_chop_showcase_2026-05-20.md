# Melodic Source-Chop Showcase

Date: 2026-05-20

## Purpose

`DH_RushArp_120_A.wav` is a useful local melodic/arp fixture, but the current
Source Timing probe reports it as `needs confirm` with unavailable grid use. It
should not be presented through the Feral grid drum-support path as if Riotbox
had trusted kick/backbeat timing.

This benchmark defines the current bounded alternative:

```bash
just melodic-source-chop-showcase /tmp/riotbox-815-melodic local-riotbox-815
```

The helper:

- runs `source_timing_probe --json` on the source and requires
  `grid_use: unavailable`
- requires `low_timing_confidence` and `weak_kick_anchor` warnings, preserving
  the no-drum-grid-trust boundary
- renders `feral_before_after_pack` for the same source window
- validates the listening manifest with existing artifact checks
- requires non-silent W-30 source-chop output and a source-vs-after delta above
  the manifest threshold

## Listening Order

For the generated pack:

1. `01_source_excerpt.wav`: raw melodic source window.
2. `stems/w30_source_chop.wav`: source-backed W-30 chop from the same material.
3. `02_riotbox_feral_changed.wav`: bounded Riotbox after render.
4. `03_before_then_after.wav`: source excerpt, silence, then after render.

## Boundary

This is not a new arranger, live TUI path, or trusted drum-grid showcase. It is a
local melodic/source-chop proof that reuses the existing before/after manifest
and output QA seam until a fuller melodic source showcase lands.
