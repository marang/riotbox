# TR-909 Kick Pressure Showcase Review - 2026-05-14

## Scope

Review note for `RIOTBOX-805`, a P013 musical-depth slice focused on making the
Feral grid TR-909 support layer add measurable kick/body pressure without
breaking source diversity or source-grid alignment.

Reviewed command:

```bash
scripts/generate_representative_source_showcase.sh /tmp/riotbox-805-showcase local-riotbox-805 4.0 4
```

Generated WAVs remain local and untracked.

## Change

The Feral grid TR-909 path now applies a bounded source-aware kick-pressure
layer after the existing source-support render:

- source profile selects pressure gain, body frequency, tail length, and click
  amount
- pressure anchors remain locked to the shared beat/bar grid
- the manifest and report expose anchor count, pressure gain, low-band RMS
  ratio, low-band delta, peak, and reason
- the musical-quality gate rejects candidates whose TR-909 pressure is missing
  or too decorative

The realtime TR-909 callback, Jam action model, Session, and `ActionCommand`
surface are unchanged.

## Evidence

The first implementation attempt was too uniform: it increased low-end body but
failed the source-diversity validator. The landed policy reduces the pressure
gain and keeps the pressure proof source-aware enough that the representative
showcase passes diversity, reproducibility, observer/audio correlation, and
musical quality.

Selected musical candidate:

- Case: `break_low_drive/head`
- Verdict: `musically_convincing_candidate`
- Full mix RMS: `0.018752`
- Low-band RMS: `0.017476`
- Generated-support/source RMS ratio: `0.351498`
- TR-909 kick-pressure low-band ratio: `1.318228`
- TR-909 kick-pressure anchors: `16`
- W-30 offbeat trigger count: `4`
- W-30 distinct bar pattern count: `4`
- W-30 unique slice offsets: `6`

## Boundary

This is still a bounded offline Feral-grid policy, not a full TR-909 synthesis
redesign or live mixer change. The useful improvement is that the current
showcase can prove drum-body pressure as an output metric while preserving the
existing timing and source-diversity gates.
