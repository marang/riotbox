# DH_BeatC Auto Feral Grid Source Timing - 2026-05-21

## Purpose

This note captures the current local DH_BeatC Feral-grid auto-BPM behavior after
the P012 Source Timing confidence work.

The source WAV is intentionally local. This is a musician-facing benchmark note
for the documented Recipe 15 variant, not a mandatory fresh-clone CI input.

## Command

```bash
just dh-beatc-auto-feral-grid-proof local-dh-beatc-feral-grid-auto-proof
```

The proof runs the equivalent of:

```bash
just feral-grid-pack "data/test_audio/examples/DH_BeatC_120-01.wav" local-dh-beatc-feral-grid-auto-proof auto 8 1.0 0.0
```

If the local DH_BeatC WAV is missing, the proof exits as a skip.

## Captured Local Result

| Field | Value |
| --- | --- |
| `grid_bpm_source` | `source_timing` |
| `grid_bpm_decision_reason` | `source_timing_needs_review_manual_confirm` |
| `source_timing.readiness` | `needs_review` |
| `source_timing.grid_use` | `short_loop_manual_confirm` |
| `source_timing.primary_bpm` | `120.185` |
| `source_timing.primary_downbeat_offset_beats` | `0` |
| `source_timing.warning_codes` | `PhraseUncertain` |
| `tr909_source_grid_alignment.hit_ratio` | `1.000` |
| `mc202_source_grid_alignment.hit_ratio` | `1.000` |
| `w30_source_grid_alignment.hit_ratio` | `0.750` |
| `source_grid_output_drift.hit_ratio` | `0.969` |

## Interpretation

- DH_BeatC auto-BPM now has the same local proof shape as Beat03.
- The grid is useful but intentionally not presented as a fully locked long
  phrase because phrase evidence is short.
- TR-909, MC-202, W-30, and the generated support mix all clear the current
  source-grid hit-ratio floor in the local proof.
