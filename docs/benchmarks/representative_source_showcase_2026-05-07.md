# Representative Source Showcase 2026-05-07

Use this local pack only after the source-diversity, mix-balance,
source-aware TR-909, and W-30 source-chop fixes have landed.

```bash
just representative-source-showcase
```

The command writes ignored artifacts under:

```text
artifacts/audio_qa/local-representative-source-showcase/
```

It generates deterministic synthetic fixture sources, renders at least five
distinct source files with multiple source windows, validates the primary-window
packs with the source-showcase diversity gate, runs a same-source reproducibility
check, writes an observer/audio correlation summary, and runs the musical-quality
review gate.

Review order for each `packs/<case>/<window>/` directory:

1. `00_source_window.wav`
2. `stems/02_w30_feral_source_chop.wav`
3. `03_riotbox_source_first_mix.wav`
4. `04_riotbox_generated_support_mix.wav`

The musical-quality summary lives at:

```text
validation/musical-quality.md
```

That summary marks one `musically_convincing_candidate` only when the pack has
audible generated support, W-30 source-chop energy, low-end support, source
anchor evidence, bounded source-first masking, non-static bar movement, and a
measured W-30 trigger-variation proof. Event density remains a guardrail, but
the W-30 offbeat-trigger, distinct-bar-pattern, and unique-slice-offset metrics
are the direct proof that a sparse source chop is not just a static repeated
audition. It does not claim automatic taste scoring or release-ready output.

Durable conclusions belong in docs or review notes. Generated WAVs remain local
and untracked.
