# Representative Showcase Musical Quality Review - 2026-05-14

## Scope

Review note for `RIOTBOX-799`, the first P013 bridge slice after the P012 Source
Timing gates became strong enough to support deeper musical-output work.

The reviewed path is the local representative showcase:

```bash
scripts/generate_representative_source_showcase.sh /tmp/riotbox-799-showcase-after local-riotbox-799-after 4.0 4
```

Generated WAVs remain local and untracked.

## Finding

The previous representative source showcase had useful technical evidence, but
the generated-support mixes were still too close to QA artifacts: source
response was measurable, while the TR-909 support was often too quiet to feel
like an intentional musical layer.

This slice adds an explicit musical-quality review gate and increases only the
generated-support mix policy. The source-first mix remains constrained by the
existing generated/source masking threshold.

Selected candidate after the change:

- Case: `tonal_hook_chop/late`
- Verdict: `musically_convincing_candidate`
- Full mix RMS: `0.018757`
- Low-band RMS: `0.017573`
- Generated-support/source RMS ratio: `0.272528`
- Source-first generated/source RMS ratio: `0.063978`
- W-30 preview RMS: `0.179584`
- Source anchors: `4`
- Full-mix bar similarity: `0.962824`

Baseline comparison for the same case from the pre-change local pack:

- Generated-support/source RMS ratio: `0.134338`
- Full-mix bar similarity: `0.985340`
- Full mix RMS: `0.021072`

## Interpretation

The improved candidate is stronger because the generated TR-909 layer is no
longer merely decorative, while the source-first proof still shows the generated
material does not mask the source. The selected candidate also keeps W-30
source-chop energy, low-end support, source-anchor evidence, and bar movement
inside explicit review thresholds.

This does not mean Riotbox has release-quality arrangement output yet. It means
the local representative showcase now has one named candidate that is suitable
for musician-facing review instead of only technical QA inspection.

## Boundary

The new gate is not automatic taste scoring. It is a product-review guardrail
that prevents future agents from presenting a technically reactive but
unconvincing pack as a musical demo without at least one explicit listening
candidate.
