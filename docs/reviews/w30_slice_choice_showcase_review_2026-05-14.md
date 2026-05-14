# W-30 Slice Choice Showcase Review - 2026-05-14

## Scope

Review note for `RIOTBOX-804`, a P013 musical-depth slice focused on making the
Feral grid W-30 source chop choose multiple source-derived offsets instead of
reusing one static read point.

Reviewed command:

```bash
scripts/generate_representative_source_showcase.sh /tmp/riotbox-804-showcase local-riotbox-804 4.0 4
```

Generated WAVs remain local and untracked.

## Change

The Feral grid W-30 path now builds a small source slice-choice plan from the
selected chop preview:

- eight candidate source regions are scored by RMS, rising-edge energy, and peak
  level
- the strongest offsets are deduplicated and used by the trigger-variation
  pattern
- the manifest and report expose unique source offsets and selected offset span
- static slice-choice collapse is rejected by the musical-quality fixture gate

The realtime W-30 callback, Jam action model, Session, and `ActionCommand`
surface are unchanged.

## Evidence

The representative showcase passes source diversity, reproducibility,
observer/audio correlation, and musical quality.

Selected musical candidate:

- Case: `break_low_drive/head`
- Verdict: `musically_convincing_candidate`
- Full mix RMS: `0.018069`
- Low-band RMS: `0.016802`
- Event density per bar: `53.750000`
- Full-mix bar similarity: `0.421471`
- Generated-support/source RMS ratio: `0.288840`
- Source-first generated/source RMS ratio: `0.067805`
- W-30 preview RMS: `0.180757`
- W-30 offbeat trigger count: `4`
- W-30 distinct bar pattern count: `4`
- W-30 unique slice offsets: `6`
- W-30 slice offset span: `1280` samples

## Boundary

This is still a bounded offline Feral-grid policy, not source separation or a
full sampler architecture. The useful improvement is that the current showcase
can now prove both trigger timing variation and slice-choice variation while
remaining deterministic and grid-aligned.
