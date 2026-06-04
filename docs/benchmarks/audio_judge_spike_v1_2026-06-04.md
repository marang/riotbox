# Audio Judge Spike v1

`riotbox.audio_judge_spike.v1` is the first P021 spike contract for comparing
Riotbox-owned deterministic audio evidence with optional CLAP/MERT-style
embedding providers.

The spike intentionally does not produce a musical-pass verdict. It reports:

- the supplied `agent-review.json` and human label corpus inputs
- a deterministic `riotbox_metrics_baseline` prediction from dense-break proof
  fields
- optional provider availability for CLAP-style and MERT-style offline QA
  experiments
- matched label examples, confusion counts, and coverage/failure examples
- a recommendation of `useful` or `not_ready`
- `human_verdict: unverified`

Current status: `not_ready`.

Why:

- the committed corpus has only fixture labels
- the fixture spike now matches pass, weak, and fail dense-break review packs,
  but those are still synthetic calibration examples
- the generated dense-break smoke currently matches one pass label
- optional embedding providers are not required in CI and are not calibrated

This is useful software work because it gives Codex and CI a concrete report
shape for judging whether an audio-judge approach is worth keeping. It is useful
for the musician because it prevents Riotbox from promoting a technically valid
loop as "good" until weak hooks, bass pressure, destructive contrast, and replay
value are checked against human listening labels.

Run:

```bash
just audio-judge-spike-fixtures
just agent-musical-review-pack-smoke
just audio-judge-spike-generated-smoke
```
