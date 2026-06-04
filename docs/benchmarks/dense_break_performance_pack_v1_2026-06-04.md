# Dense-Break Performance Pack v1

`riotbox.dense_break_performance_pack.v1` is the first controlled Riotbox
sound-quality Golden Path.

It renders an 8-bar source-backed rave-punk break performance from a local
dense-break source, currently `data/test_audio/examples/Beat03_130BPM(Full).wav`.
The source file is a short loop; the pack treats it as source material and
arranges a longer performance from generated Riotbox stems.

Target structure:

- Bars 1-2: source character plus W-30-style chop motif.
- Bars 3-4: W-30 source chop becomes the main hook.
- Bars 5-6: TR-909 and MC-202 pressure lift.
- Bar 7: dropout followed by source-chop stutter.
- Bar 8: restore hit where break transient and bass pressure land together.

Generated artifacts:

- `00_source_window.wav`
- `01_chop_hook.wav`
- `02_pressure_lift.wav`
- `03_dropout_stutter.wav`
- `04_restore_hit.wav`
- `05_full_performance.wav`
- `performance-report.json`
- `agent-review.json`
- `agent-review.md`
- `visuals/*.waveform.png`
- `visuals/*.spectrogram.png`
- `README.md`

The report emits `agent_verdict: agent_promising` only when the pack avoids the
known bad-output modes for this Golden Path: weak W-30 hook presence, missing
bass-pressure lift, weak dropout/stutter contrast, weak restore transient,
near-static bars, source-copy collapse, or buried bass pressure.

`agent_promising` is not a final musical pass. The report must keep
`human_verdict: unverified` until a structured listening review or the future
P021 calibrated audio judge supplies stronger verdict evidence.

Run:

```bash
just dense-break-performance-pack
just dense-break-performance-pack-smoke
just agent-musical-review-pack-smoke
```
