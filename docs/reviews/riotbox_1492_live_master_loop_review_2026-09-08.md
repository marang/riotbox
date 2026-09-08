# RIOTBOX-1492 — Exact live master loop review

Status: exact take-02 composite loop accepted by human review

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

## Source-free correction and take-02 admission

On resumed work, the existing `cpal_spike` produced 25 silent callbacks.
Retained snapshots show `alsa_playback.cpal_spike` instead of the requested
custom node name; null-sink target and all three safety properties remain
present. This reproduces the old selector's false negative.

The corrected operator selector binds Client PID to the launched process,
follows Node client ID, requires the exact null-sink serial, two active stereo
links and all three safety properties, and rejects outgoing null-sink links.
Seven snapshots pass; wrong PID and four unsafe routing mutations are rejected.
No product code, audio algorithm or threshold changed.

The resumed request continues with one replacement take under
`artifacts/development/riotbox-1492/take-02`, from the same original qualified
Session and permitted audio inputs. A fresh bounded access log precedes reads.
Preserve all source, capture, timing and listening boundaries above. Persist
actual-route snapshots throughout the invocation, including unmatched objects.
Take-01 remains immutable failed evidence. This corrects operator evidence,
not the musical candidate or a product gate.

## Take-02 technical result

The unchanged V2 recorder captured beat 4 through beat 12 at 120 BPM, exactly
192,000 stereo float32 frames at 48 kHz. Product proof, WAV payload/file hashes,
receipt gates, Session identity and stopped runtime agree. The external observer
now retains the process-owned CPAL stream and active links to the isolated null
sink. File-open tracing shows one registered source-owner read and one copied
capture read, no original-source or Holdout access. The temporary sink was
removed after capture; the physical default route was unchanged. This is real
callback evidence through a virtual endpoint, not a hardware/acoustic test.

- WAV SHA-256: `8d598b852a2811c7614de8755d7a9da870339b75582a922510e6c82b1d77f4c0`
- Proof SHA-256: `2ae77d457959cc31093c604ddeddc6a2b23665d2e0d300a2de67bd01f995a622`
- Exact two-repeat presentation SHA-256:
  `61832723eaa24abc79c645e4c6a978f1428e056ed359964e5fc1904c4038a4fa`
- Presentation: `artifacts/development/riotbox-1492/take-02/loop-review-2x.wav`,
  384,000 frames / 8.000 seconds, no gain/fade/resampling or recipe change.
- Original and presentation: FFmpeg integrated loudness -17.1 LUFS, estimated
  true peak -6.2 dBTP; sample peak -8.431 dBFS, RMS -19.612 dBFS, zero clips.
- Stereo channels are sample-identical. DC per channel is -0.002569.
- Raw end/start sample step is 0.007979 per channel, below internal adjacent
  difference p99 0.053636 and maximum 0.446067. This descriptive comparison does
  not establish an inaudible seam or award loopability.

Take-02 WAV bytes happen to match take-01. Neither has been heard, and the
failed take-01 route record is not promoted by that identity. Only take-02 owns
the new completed route proof. The review pack under
`artifacts/audio_qa/local/listening-reviews/RIOTBOX-1492` remains explicitly
`human_verdict: unverified` until fresh readiness, bounded playback and feedback.

Contributor assignment: this is a **composite runtime master**, not the
isolated V4 semantic W-30 export. The restored Session activates W-30 live recall
and TR-909 `break_reinforce` support. TR-909 does not require a non-null pattern
reference to sound: `render_tr909_buffer` gates transport/mode/tempo, and
`should_trigger_step` enables `BreakReinforce` steps without pattern adoption.
Its existing fixed renderer support is not source-composed drum intelligence.
MC-202 is idle/silent, Source Monitor is `riotbox_only` (raw source excluded),
and the resample tap is idle. The recorder adds no sound processing beyond the
existing master path. The earlier isolated W-30 keep cannot be transferred to
this composite. Review loop reuse and transition, not new hardness or bass
ownership.

Preflight report SHA-256:
`87cb3f0800821583bc1d51314fecaceb1ab5e2a337a337d6ee5f7bef1aca5c86`.
Actual-route snapshots SHA-256:
`d566e300ab22aec2900992639257085a1381cfb58924401381842aa25b33b082`.

Independent artifact-bound agent assessment completed before listener readiness;
local `agent-prelisten.md` SHA-256:
`49586109f89fedacd6a3f2f8e6a912fb9db0fb04a44e534cfab6e9d42d6536bf`.
It confirms the composite assignment and separates technical validity from
predicted musical usefulness. Its prediction is retained unchanged for later
comparison with the listener, not used as a human verdict. Focused
`just live-master-recording-contract-smoke` passes. At that point, no human
playback had occurred.

## Human verdict and scope

After fresh explicit readiness, the exact eight-second presentation played once
through the physical default output. Playback exited normally; subsequent
PipeWire inspection found no remaining review stream. Other desktop audio was
not modified or claimed silent. The listener asked whether the beat inside the
hook was the target. The review scope was clarified as the entire recorded
hook-plus-beat loop, not isolated drums or hardness; no redundant replay followed.

The listener accepted it: the sound was good and hook/beat were well matched.
Record `human_verdict: keep` for this exact composite loop and its bounded reuse
question. `strongest_element: none` means no strongest element was specified;
`hook_after_two_bars: inconclusive` means hook strength was not separately rated;
`source_recognition: not_applicable` reflects the absence of a source A/B.
No explicit click/seam rating was supplied. Those dimensions are not invented;
the raw seam measurements remain technical evidence only. The prior agent
prediction stays unchanged rather than being rewritten to agree with feedback.

- Structured review: `artifacts/audio_qa/local/listening-reviews/RIOTBOX-1492/review.json`
- Structured review SHA-256:
  `ae8bd981061625ab24fc3f3101718d451d4a121c5bc22f78b41f83138a22a18d`
- Summary SHA-256:
  `7a015525f0cf6f8f0091b56f4edfbb2d522585b69e70f2f7e10922699278f42c`

This completes one exact V2 live-recording loop review. It changes no product
code, sound recipe, thresholds, Session schema, TUI, or export behavior. It does
not prove isolated W-30 quality, source-composed TR-909 intelligence, hardness,
bass pressure, source generality, physical capture-device endurance, DAW import,
demo-bank promotion, or overall release readiness. RIOTBOX-1036 remains the
broader recording/export anchor; RIOTBOX-1491 and TUI remain deferred.
