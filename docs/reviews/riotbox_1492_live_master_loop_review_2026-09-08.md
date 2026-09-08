# RIOTBOX-1492 — Exact live master loop review

Status: take-01 stopped fail-closed at external route-evidence gate; human verdict unverified

## Bounded purpose

Follow RIOTBOX-1486 with one assessment of the existing
`runtime_master_bar_window_v2` recording as a reusable two-bar loop. This is
P016 workflow qualification under RIOTBOX-1036, not a new musical mechanism,
TUI, recorder, algorithm, threshold, or source-family qualification.

The old RIOTBOX-1486 V2 hashes remain historical technical evidence. Its exact
WAV/proof was not found at the inspected local artifact locations. The available
`local-dense-break-live-path-smoke` Session is synthetic and has been regenerated
by CI, so it cannot stand in for this Development review.

## Selected existing product state

Use only `tonal_rusharp_120`, the already-qualified W-30 semantic hook state:

- Registry: `docs/benchmarks/sound_excellence_source_corpus_v1.json`.
- Registered original: `data/test_audio/examples/DH_RushArp_120_A.wav`.
- Original/owner-copy SHA-256:
  `ec2a0c930eb338bf81cd5cb4b5fef487e07c140ad40181e1d92b2a0990334e0e`.
- Prior state directory:
  `artifacts/development/riotbox-1482/product-qualification-2026-08-28-final-a/tonal_rusharp_120`.
- Input metadata: `session.json` and `source-graph.json` in that directory.
- Permitted audio inputs: its exact `registered-source-owner-copy.wav` and
  Session-owned `captures/cap-01.wav`; no other capture or source.
- Source, Session, capture window and confirmed manual 120 BPM grid must agree.
- Preserve the prior state. Work on a fresh Session/Graph/capture copy under
  `artifacts/development/riotbox-1492/take-01`; explicit copied Graph override
  prevents publication into the old evidence directory.

Before those two selected audio reads, create a fresh bounded access log and
check registered identity/path/hash exclusion against protected partition
metadata. No original source read, source-directory discovery, Holdout,
commercial audio, source substitution, new analysis or recipe tuning is allowed.
The unchanged source-owner copy is read once for identity admission and once
by the normal product restore hash/decode gate. The capture is read once for
copying and once from the new Session directory for runtime hydration.
Record the exact operations and hashes; stop on any contract mismatch.

## Capture and human safety

One take only through the existing live-master CLI, with a verified isolated
silent output endpoint and bounded process lifetime. Preserve the real CPAL
post-limiter callback tap; never label an offline render as a live capture.
A virtual host endpoint is not a physical-device or acoustic-output test.
Do not change the user's default sink, volume, or unrelated audio streams.
If isolation cannot be verified, stop before starting the runtime.

Preflight the exact resulting float32 WAV and its proof/Session/observer refs:
format, frames, eight-beat timing, fault counters, peak/RMS/loudness/true peak,
silence, clipping and the raw end-to-start discontinuity. Inventory all actual
contributors from restored runtime state rather than assuming that a master
recording is W-30-only. No crossfade or gain adjustment may conceal a bad loop.

For one boundary audition, concatenate exactly two unprocessed copies into an
eight-second presentation at 120 BPM. Verify sample identity and technical
safety of that exact presentation. An independent agent assessment precedes
fresh explicit listener readiness. Playback must end and silence be verified.
The question is loop reuse, groove and transition quality, not new sound,
hardness, general quality or release readiness. Prior hook keeps are context,
not a verdict on this new capture. No alternate candidate without a documented
reason and renewed scope review.

## Take-01 result

The registered owner-copy hash matched. The one copied capture SHA-256 was
`8460fd78a74fb1290555413ffd8160262ce88077c55fbdf09e417233d8d1f183`.
File-open tracing records exactly one runtime source-owner read and one copied
capture hydration read; no original source, other source or Holdout was opened.

One ordinary CLI invocation wrote 192,000 stereo float32 frames at 48 kHz and
120 BPM. It requested beat 4 and reports capture through beat 12 (floating-point
roundoff only), stopped runtime, and three passed product receipt gates.
These values are reported product evidence, not independent WAV preflight or
a human verdict. Reported WAV SHA-256:
`8d598b852a2811c7614de8755d7a9da870339b75582a922510e6c82b1d77f4c0`;
reported proof SHA-256:
`d68e69e51664dc555c94c8bc5f6beeb47ebb360de16e31e726dbe1ea90fe21d8`.

The separate operator safety gate **failed**: the polling observer did not
record an active link for the expected runtime node name. A prior zero-only
ALSA probe under the exact per-process configuration had demonstrated target
serial 2877, null-sink driver 58 and no-fallback/no-reconnect/no-move properties.
That probe is not a substitute for the missing actual-take route observation.
The successful recorder status therefore cannot promote this take to listening.
This does not establish a recorder defect or prove physical audio leakage;
the actual-take routing evidence is incomplete.

No second take, waveform analysis, review presentation or playback followed the
failed gate. The temporary null sink (module 536870916) was removed after the
runtime exited. The physical default sink stayed unchanged. All prior evidence
and take-01 files remain in place, including the committed take-01 Session,
`access-log.ndjson`, `file-access.trace`, `capture-summary.json`,
`capture-stderr.log`, `observer.ndjson`, WAV and proof. The failed access log is
not rewritten into success. The next bounded action is source-free diagnosis
of route observation; any new take needs an explicit new attempt record before
audio access. Musical usefulness remains unverified.

Frozen local failure evidence SHA-256:

- access log: `a5505a41c5596c5101b226063543db65dce96cd0c392490e05087b662eac6823`
- capture summary: `31f0b71d1dc97b8e92b8f6b101a088d7248dea2cb5ba3fe9b6b642f69b55e6b7`
- file-open trace: `8ef7ebf95a930cc636e09178895d53d2f0fe15a96e4d8fc702d6930d4b9cc357`

Independent metadata review confirmed that success of the recorder cannot
replace the missing routing observation. The selector depended on an exact
node name and did not retain unmatched snapshots. The specific cause remains
unresolved: a proposed explanation that `PIPEWIRE_PROPS` cannot affect ALSA
streams contradicts the observed zero-only aplay probe, which did expose the
custom node name and all three requested safety properties. Do not present that
explanation as a diagnosed cause. Before any renewed take, use a source-free
CPAL silence probe to validate stream identification and retained route evidence.
