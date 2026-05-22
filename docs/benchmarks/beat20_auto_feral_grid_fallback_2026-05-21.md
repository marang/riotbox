# Beat20 Auto Feral Grid Fallback - 2026-05-21

## Purpose

This note captures the current local Beat20 Feral-grid auto-BPM fallback
behavior after `RIOTBOX-862`.

Beat20 now has useful BPM and beat evidence, but competing downbeat phases keep
it out of the Source Timing auto-grid path. This is the negative proof paired
with the Beat03, Beat08, and DH_BeatC positive short-loop proofs.

The source WAV is intentionally local. This is a musician-facing benchmark note,
not a mandatory fresh-clone CI input.

## Command

```bash
just beat20-auto-feral-grid-fallback-proof local-beat20-feral-grid-auto-fallback-proof
```

The proof runs the equivalent of:

```bash
just feral-grid-pack "data/test_audio/examples/Beat20_128BPM(Full).wav" local-beat20-feral-grid-auto-fallback-proof auto 8 1.0 0.0
```

If the local Beat20 WAV is missing, the proof exits as a skip.

## Captured Local Result

| Field | Value |
| --- | --- |
| `grid_bpm_source` | `static_default` |
| `grid_bpm_decision_reason` | `source_timing_requires_manual_confirm` |
| `source_timing.readiness` | `needs_review` |
| `source_timing.grid_use` | `manual_confirm_only` |
| `source_timing.primary_bpm` | `128.397` |
| `source_timing.primary_downbeat_offset_beats` | `2` |
| `source_timing.downbeat_status` | `ambiguous` |
| `source_timing.primary_downbeat_score` | `0.27302644` |
| `source_timing.primary_downbeat_margin` | `0.0053614676` |
| `source_timing.alternate_downbeat_phase_count` | `3` |
| `source_timing.confidence_result` | `candidate_ambiguous` |
| `source_timing.alternate_evidence_count` | `6` |
| `source_timing.warning_codes` | `PhraseUncertain`, `AmbiguousDownbeat` |
| `tr909_source_grid_alignment.hit_ratio` | `1.000` |
| `mc202_source_grid_alignment.hit_ratio` | `1.000` |
| `w30_source_grid_alignment.hit_ratio` | `0.750` |
| `source_grid_output_drift.hit_ratio` | `1.000` |

## Interpretation

- Beat20 auto-BPM does not use Source Timing as the grid source while downbeat
  phase remains ambiguous.
- The source timing evidence is still useful to display: BPM and beat evidence
  are stable, but the grid remains manual-confirm-only.
- This preserves the musician-facing contract: Riotbox can explain uncertainty
  without falsely locking generated support to an ambiguous bar phase.
