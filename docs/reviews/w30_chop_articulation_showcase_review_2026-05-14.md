# W-30 Chop Articulation Showcase Review - 2026-05-14

## Scope

Review note for `RIOTBOX-802`, a P013 musical-depth slice focused on making the
representative showcase W-30 source chop read more like an intentional gesture
and less like a flat technical audition.

Reviewed command:

```bash
scripts/generate_representative_source_showcase.sh /tmp/riotbox-802-showcase-4 local-riotbox-802-4 4.0 4
```

Generated WAVs remain local and untracked.

## Change

The Feral grid W-30 source-chop preview now applies:

- a short attack and decaying chop envelope
- transient emphasis before gain and edge fade
- a tighter maximum normalization gain so weak high-frequency material is not
  lifted until it masks source identity
- manifest metrics for W-30 body RMS, tail RMS, and tail/body ratio

The global W-30 runtime contract is unchanged. This only affects the
`feral_grid_pack` source-chop preview path.

## Evidence

The representative showcase still passes source diversity, reproducibility,
observer/audio correlation, and the musical-quality gate.

Selected musical candidate remains:

- Case: `tonal_hook_chop/late`
- Verdict: `musically_convincing_candidate`
- Full mix RMS after: `0.018602`
- Full-mix bar similarity after: `0.960649`
- Generated-support/source RMS ratio after: `0.274855`
- Source-first generated/source RMS ratio after: `0.064523`
- W-30 preview RMS after: `0.172702`
- W-30 body RMS after: `0.286206`
- W-30 tail RMS after: `0.077223`
- W-30 tail/body ratio after: `0.269817`

Comparison against the RIOTBOX-799 local candidate:

- W-30 preview RMS before: `0.179584`
- W-30 render RMS before: `0.006443`
- Full-mix bar similarity before: `0.962824`
- Generated-support/source RMS ratio before: `0.272528`

## Interpretation

This is a controlled articulation improvement, not a claim that W-30 chopping is
finished. The useful change is that the source chop now has a measurable body
and tail contour while the full showcase still preserves source diversity and
the source-first masking boundary.

Next musical work should move from preview shaping toward more expressive W-30
trigger patterns, slice choices, and phrase-level variation.
