# Audio Output QA

Use this incident procedure when Riotbox output is wrong, silent, identical,
fallback-like, placeholder-like, or weaker than the control path claims.
The [audio-QA router](../../../../docs/specs/audio_qa_workflow_spec.md) remains
the canonical product gate.

## Evidence Packet

1. Reproduce the exact source, command, seed/config, transport state, and user
   gesture. Preserve the command and artifact path.
2. State the expected audible behavior in one musician-facing sentence.
3. Record control-path context: action, queue/commit, transport position,
   selected source/policy, provenance, logs, and state transition.
4. Render or capture the nearest deterministic downstream PCM/WAV seam. If the
   claim includes the live device or mixer, also prove that exact path.
5. Analyze the exact artifact, not a sibling report:
   - inspect format, rate, channels, duration, frames, and container
   - measure peak, RMS/loudness, DC, silence, clipping, and channel balance
   - inspect waveform/onsets, loop boundaries, repeated transients, and timing
   - compare with the relevant baseline/control using time-local and
     role-appropriate spectral evidence; hashes are only a duplicate shortcut
6. Interpret the measurements in prose. Say whether they support the report,
   contradict it, or reveal a different failure.
7. Convert the finding into one concrete implementation, audio-policy,
   regression, threshold, or musician-understanding follow-up.

Use `ffprobe`, `ffmpeg` signal analysis, and a waveform/comparison tool such as a
project helper, `sox`, Python `wave`/`numpy`, or a DAW/spectrogram. If a required
tool is unavailable, name the missing evidence and use the nearest valid
substitute; never replace output proof with more log inspection.

## Judgment Rules

- Control proof and output proof are independent. Neither substitutes for the
  other.
- Source-backed behavior needs a source-vs-control comparison that can detect
  silence, fallback collapse, or source-insensitive output.
- Compare the locally changed gesture/window as well as the whole render.
- Interpret level, timing, spectral, and correlation evidence for the claimed
  role; raw tool output is not a conclusion.
- If metrics pass but the musician still hears a weak result, the feature is
  technically partial.
- Prefer fixing observable sample, drum, bass, mix, trigger, or arrangement
  behavior before building another report or validator.
- Freeze the smallest regression that catches the observed failure after the
  behavior is understood.

Do not answer musician feedback with “the internal path works.” The audible
artifact and, when in scope, the live product path are part of the contract.
