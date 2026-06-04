# Human Listening Label Corpus v1

`riotbox.human_listening_label_corpus.v1` is the first versioned label contract
for calibrating Riotbox audio-judge work against human taste.

It is separate from `riotbox.listening_review.v1`:

- `listening_review.v1` records one structured review pack/verdict.
- `human_listening_label_corpus.v1` collects stable labels for calibration and
  later judge evaluation.

Each label records:

- `human_verdict`: `pass`, `weak`, `fail`, or `inconclusive`
- source family and source id
- review pack schema/id
- artifact identities by SHA-256, so labels attach to generated review packs
  without committing local source audio
- reason tags for hook clarity, hardest hit, bass pressure, destructive
  contrast, source character, and replay value after eight bars
- summary, avoid-list, and for weak/fail labels a failure reason and preferred
  direction

The first fixture corpus covers dense-break pass, weak, and fail labels plus
tonal-hook and sparse-bass-pressure calibration examples for the agent musical
review pack shape. It is schema and calibration-shape evidence, not a real taste
corpus yet.

Structured listening reviews can be imported into the same corpus shape with
`scripts/import_listening_review_label.py` when the review carries explicit
`audio_judge_label` metadata: source family/id, review pack identity, artifact
hashes, created date, and reason tags. The importer maps
`keep -> pass`, `technically_ok_but_musically_weak -> weak`, `reject -> fail`,
and `inconclusive -> inconclusive`; it rejects `unverified` reviews and missing
metadata.

Run:

```bash
just human-listening-label-corpus-fixtures
just listening-review-label-import-fixtures
```
