# Beat08 Auto Feral Grid Source Timing - 2026-05-21

## Purpose

This note captures the current local Beat08 Feral-grid auto-BPM behavior after
the P012 Source Timing confidence work.

Beat08 remains the easiest documented source for first timing and queue/commit
workflows, so its `auto` Feral-grid behavior should stay pinned alongside
Beat03 and DH_BeatC.

The source WAV is intentionally local. This is a musician-facing benchmark note
for the documented Recipe 15 variant, not a mandatory fresh-clone CI input.

## Command

```bash
just beat08-auto-feral-grid-proof local-beat08-feral-grid-auto-proof
```

The proof runs the equivalent of:

```bash
just feral-grid-pack "data/test_audio/examples/Beat08_128BPM(Full).wav" local-beat08-feral-grid-auto-proof auto 8 1.0 0.0
```

If the local Beat08 WAV is missing, the proof exits as a skip.

## Captured Local Result

| Field | Value |
| --- | --- |
| `grid_bpm_source` | `source_timing` |
| `grid_bpm_decision_reason` | `source_timing_needs_review_manual_confirm` |
| `source_timing.readiness` | `needs_review` |
| `source_timing.grid_use` | `short_loop_manual_confirm` |
| `source_timing.primary_bpm` | `128.397` |
| `source_timing.primary_downbeat_offset_beats` | `3` |
| `source_timing.warning_codes` | `PhraseUncertain` |
| `tr909_source_grid_alignment.hit_ratio` | `1.000` |
| `mc202_source_grid_alignment.hit_ratio` | `1.000` |
| `w30_source_grid_alignment.hit_ratio` | `0.750` |
| `source_grid_output_drift.hit_ratio` | `0.969` |

## Interpretation

- Beat08 auto-BPM is Source Timing-driven with visible manual-confirm caution.
- The grid is useful but still not presented as a fully locked long phrase
  because phrase evidence is short.
- TR-909, MC-202, W-30, and the generated support mix all clear the current
  source-grid hit-ratio floor in the local proof.
