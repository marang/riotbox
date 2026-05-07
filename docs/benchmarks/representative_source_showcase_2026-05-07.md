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
check, and writes an observer/audio correlation summary.

Review order for each `packs/<case>/<window>/` directory:

1. `00_source_window.wav`
2. `stems/02_w30_feral_source_chop.wav`
3. `03_riotbox_source_first_mix.wav`
4. `04_riotbox_generated_support_mix.wav`

Durable conclusions belong in docs or review notes. Generated WAVs remain local
and untracked.
