# Agent Musical Review Pack v1

`riotbox.agent_musical_review_pack.v1` is the first agent-facing review layer
for the dense-break sound-quality Golden Path.

It is not a model-based musical-pass oracle. It packages the audio, deterministic
metrics, verdict fields, and visual evidence that an agent can inspect before
deciding whether a render is `agent_fail`, `agent_weak`, or
`agent_promising`.

The current implementation is produced by the dense-break performance pack and
writes:

- `agent-review.json`
- `agent-review.md`
- waveform PNGs for source window, chop hook, pressure lift, dropout/stutter,
  restore hit, and full performance
- spectrogram PNGs for the same roles
- links back to `performance-report.json`

Verdict discipline:

- `agent_fail`: multiple weak-output guards failed.
- `agent_weak`: the pack rendered, but one or two musical guards failed.
- `agent_promising`: known weak-output modes were not caught.
- `human_verdict` remains `unverified` until structured listening review or a
  future calibrated P021 judge supplies stronger evidence.

Run:

```bash
just agent-musical-review-pack
just agent-musical-review-pack-smoke
```
