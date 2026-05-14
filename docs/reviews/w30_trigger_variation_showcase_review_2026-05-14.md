# W-30 Trigger Variation Showcase Review - 2026-05-14

## Scope

Review note for `RIOTBOX-803`, a P013 musical-depth slice focused on making the
Feral grid W-30 source chop less pattern-static without creating a second W-30
runtime path.

Reviewed commands:

```bash
scripts/generate_representative_source_showcase.sh /tmp/riotbox-803-showcase-5 local-riotbox-803-5 4.0 4
```

Generated WAVs remain local and untracked.

## Change

The `feral_grid_pack` W-30 source chop now uses a bounded trigger-variation
policy:

- beat-anchored triggers still land on the source grid
- one offbeat trigger pattern per bar adds source-backed movement
- per-bar source offsets and velocities vary the chop without changing the
  runtime W-30 contract
- manifest metrics expose trigger count, beat anchors, offbeats, skipped beat
  anchors, distinct bar patterns, and quantized-grid offset

The Realtime W-30 callback and Jam action model are unchanged.

## Evidence

The representative showcase passes source diversity after the trigger variation
change, and the musical-quality validator selects:

- Case: `tonal_hook_chop/head`
- Verdict: `musically_convincing_candidate`
- Full mix RMS: `0.028983`
- Low-band RMS: `0.027200`
- Event density per bar: `54.500053`
- Full-mix bar similarity: `0.380665`
- Generated-support/source RMS ratio: `0.171260`
- Source-first generated/source RMS ratio: `0.040204`
- W-30 preview RMS: `0.169206`
- W-30 offbeat trigger count: `4`
- W-30 distinct bar pattern count: `4`
- W-30 max quantized offset: `0.0 ms`

The unit regression compares the new varied render against the legacy static
render and verifies that the W-30 stem remains inside the source-grid drift
budget while reducing bar similarity.

## Boundary

This is not a full arranger, Ghost decision, or release-quality sampler. It is a
small source-backed offline policy that gives the current showcase a measurable
W-30 trigger-variation proof. Later work still needs live pad-bank variation,
runtime-facing pattern control, and musician-facing TUI explanations.
