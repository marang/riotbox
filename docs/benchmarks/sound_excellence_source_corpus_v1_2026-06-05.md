# Sound Excellence Source Corpus v1

Date: 2026-06-05

This corpus is a CI-safe contract for the local real-source files used by P023
sound-product work. The WAV files stay ignored under `data/test_audio/examples`;
the committed artifact is the metadata that tells agents which source families
must be represented before calling Riotbox broadly good.

The machine-readable manifest lives at:

```text
docs/benchmarks/sound_excellence_source_corpus_v1.json
```

It currently covers:

- dense breaks: `Beat03_130BPM(Full).wav`, `Beat08_128BPM(Full).wav`
- sparse drums: `DH_BeatC_KickSnr_120-01.wav`
- tonal riffs: `DH_RushArp_120_A.wav`
- pad / noise material: `DH_Fadapad_120_A.wav`
- weak source material: `Beat20_128BPM(Full).wav`
- bad-timing policy material: `DH_Fadapad_120_A.wav`

Each entry records the expected musical payoff, likely failure modes, timing
expectation, local-use / license boundary, and the QA or future pack it should
feed. This is not a quality proof and carries `human_verdict: unverified`; it is
the source-family coverage map that later rendered packs and listening reviews
must satisfy.

Validation:

```bash
just sound-excellence-source-corpus-fixtures
```

Local file-presence validation is optional because GitHub CI does not carry the
ignored WAV files:

```bash
python3 scripts/validate_sound_excellence_source_corpus.py \
  --require-existing-source-files \
  docs/benchmarks/sound_excellence_source_corpus_v1.json
```
