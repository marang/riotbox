# RIOTBOX-1402 Human Listening Review

Date: 2026-07-18
Reviewer: Markus
Verdict source: real user-session playback through the configured PipeWire
output

## Evidence Boundary

The reviewed candidate is the source-backed eight-bar Feral Break Alpha v2
render from the exact `RuntimeMix` offline simulation:

- Source: `Beat03_130BPM(Full).wav`
- Artifact: `alpha/05_feral_break_alpha_eight_bar.wav`
- Artifact SHA-256:
  `2d97345be46eb8c35e90d4018e964237f4a325713bc180e52a863f8ca6ee5117`
- Local review pack: `/tmp/riotbox-1402-bite-v2/beat03`

The artifact exercises the production mixer implementation but was not captured
from the interactive audio-device path. Its generated manifest therefore
correctly remains `quality_proof: false` and `human_verdict: unverified`.
This review records human product direction separately and does not upgrade
those machine-readable claims.

## Technical Preflight

- The exact artifact was assigned before playback.
- Peak was approximately -1.44 dBFS, with no clipping or limiter activation.
- Full-mix RMS was approximately -21.89 dBFS.
- W-30 source-backed bite increased high-frequency edge energy without changing
  the clean `grit = 0` path.
- The typed bass owner was `unassigned`; absent bass pressure was therefore not
  a failure criterion for this review.
- A four-source exact-path diagnostic matrix passed without collapsing to one
  timing envelope. Its maximum time-normalized envelope correlation was
  `0.492000`; this is diversity evidence, not musical quality proof.

## Human Verdict

Human direction verdict: **accepted for this iteration**

Reviewer summary: the iteration was audibly improved enough to keep without
further parameter tuning.

The candidate is audibly improved enough to freeze rather than continue
parameter iteration. This is not a claim that Riotbox has reached musical-alpha
quality, nor a final structured pass for the interactive live instrument path.
The accepted sound must not be changed during ticket closeout unless a
correctness defect requires it; an audible change would require fresh listening.

## Exact Live Stop Follow-Up

On 2026-07-19 the frozen session was loaded through the real interactive
PipeWire/device path with source monitoring disabled and only the W-30 raw
capture audition plus internal resample tap active. The source capture was
`cap-01`, duration `1.842116` seconds, SHA-256
`19100072eca31f94c76c1ce941521881d200f8598cc58cae6d4ed7836db70e8e`.

The bounded stop proof recorded:

- observer start with `is_playing: true`
- `Space` / `toggle_transport` after `2.953` seconds
- the same observer event reported `is_playing: false` and
  `status_message: transport paused`
- intentional `q` after a further `1.001` seconds
- the persisted Session retained `is_playing: false`
- no Riotbox runtime remained active after exit

Markus confirmed that the output became completely silent after the announced
stop. This is a real human and exact-device-path pass for transport-stop
semantics. It does not by itself upgrade the musical verdict or prove the
remaining preset, gesture, restart, recall, and trigger journey.

## Hook Isolation Correction

A later five-second playback was introduced as an isolated W-30 hook, but the
technical contributor inventory was incomplete: the source-backed promoted Pad
played together with the synthetic internal resample-tap voice. Markus heard
"ein brummen im hintergrund und den beat".

The beat was the intended source-backed W-30 Pad. The hum was not part of the
requested hook role: an ordinary `CaptureType::Pad` with empty lineage and
generation depth zero had been projected as `CaptureLineageReady`, producing a
promoted-profile oscillator near 177 Hz without a committed
`promote.resample`. RBX-152 corrects that activation contract so normal Pad
capture, promotion, recall, and trigger keep the tap idle/silent.

The corrected exact eight-bar candidate is
`e885a0641d3210a38b099c5133f0f7e801c00ab5cce0a3e27b79e0e8b41e83dc`.
It passes the exact diagnostic path without clipping or limiter dependency, but
its human verdict is unverified. The earlier accepted-for-iteration verdict
continues to describe only its exact historical hash and cannot be transferred
to the corrected render.

On 2026-07-19, the corrected binary replayed the exact persisted session for a
bounded five-second real-device check. Observer evidence in
`events-hook-no-hum-2.ndjson` recorded W-30 `live_recall` of `cap-01`, internal
resample tap `idle`, MC-202 `silent`, transport stopped, 425 audio callbacks,
and intentional quit. No Riotbox process remained afterward. Markus confirmed
the reported background hum was gone: "ja ist verschwunden". This is a human
pass for the isolation correction, not yet a new musical-taste verdict for the
corrected hook hash.

## Remaining Gate

RIOTBOX-1402 still needs exact interactive live-path verification of the frozen
behavior. Capture material, preset activation, Pad promotion/trigger, and exact
transport stop are now technically proven. The
remaining check should verify the gesture/mixer arc and intentional
quit/restart/recall/trigger behavior without using this offline render as a
substitute. The next playback request must identify the exact live candidate,
state the intended role, and wait for fresh listener readiness.

## Exact-Path Assignment Correction

On 2026-07-21 the exact diagnostic path was corrected after Markus described
the Fill sequence as an unharmonic, pasted-on composite. The old candidate had
three assignment defects: raw Source and the promoted W-30 capture were doubled
from different source phases, QA silently prepared TR-909 outside the documented
musician recipe, and the executable gesture order was `w -> f -> s` while the
Alpha recipe promised `w -> s -> f`.

Feral Break Alpha v2 now starts its performance candidate in `Riotbox` monitor
mode with typed `BreakReinforce` readiness. The exact path is
`w -> s -> f -> y -> Y+D`. The changed return separately proves that Scene
restore recovers the pre-jump projection and that the same-boundary W-30 damage
action, not unrelated lane drift, creates the additional audible change.

Technical evidence is green for the generated 132 BPM source and for real
Beat03. A real-source matrix across Beat03, Beat08, Beat20, and DH BeatC passes
without clipping or limiter activity; its maximum normalized W-30 hook-envelope
correlation is `0.494912`. These are diagnostic anti-collapse and attribution
proofs, not a human musical pass. The corrected candidate remained
`human_verdict: unverified` until the fresh bounded playback recorded below.

## Branch Review And Final Technical Gate

The branch-level Rust review found and fixed four correctness issues before the
final listening request: V2 had rewritten the historical V1 Fill focus, the
observer exercised a different gesture order than the exact renderer, adding
changed-return attribution exposed a main-thread stack overflow from large
inline render plans, and the observer/audio reuse gate imposed a generated
132-BPM fixture constant on real-source manifests. V1/V2 focus is now versioned,
the observer and renderer both execute `w -> s -> f -> y -> Y+D`, transition
plans are boxed, and reused manifests validate against their stored timing
identity.

The final technical gates pass: workspace tests, formatting, Clippy with denied
warnings, the complete audio-QA suite, generated exact smoke, Beat03
observer/audio correlation, and the four-source exact matrix. Every matrix Fill
peaks at `0.916060` before the clean-path `0.92` threshold with zero limiter
samples. `live_flow.rs` and `manifest.rs` remain above the module-policy size
guidance, but they already own distinct exact-path orchestration and manifest
validation concerns; a mechanical split during this audible behavior correction
would increase review risk and is not required for this slice.

## Exact Callback-Render Human Pass

On 2026-07-21 Markus reviewed the bounded final Beat03 performance sequence
through the configured PipeWire output:

- Artifact: `gestures/06_live_sequence.wav`
- Duration: `9.210417` seconds
- SHA-256:
  `e55b010cad7e080a6eaba53a85109b650be462c9540e95e1b2b858b1bbc2ccb8`
- Source SHA-256:
  `e752819f53f7147c2a3e3de307775f21b6bc295332b3010b13479ae7e19ae30a`
- Exact gesture arc: `w -> s -> f -> y -> Y+D`
- Monitor path: `Riotbox`; no raw Source layer
- Typed bass owner: `unassigned`; bass pressure was not a review target

The structured review is stored locally at
`artifacts/audio_qa/local/listening-reviews/RIOTBOX-1402-e55b010c/` and validates
as `riotbox.listening_review.v1` with:

- `human_verdict: keep`
- strongest element: `silence`, representing the praised breaks and pauses
- source recognition: `source_transformed_but_present`
- hook after two bars: `clear`

Reviewer summary: the complete render was musically successful, with effective
breaks and pauses, coherent layer interaction without overcrowding, and audible
aggression. The reviewer judged it to be moving in the intended direction,
formally confirmed `keep`, described the source as clearly transformed but
still recognizable, and confirmed that the hook arrived within the first two
bars.

This is a musician-facing pass for this exact callback-render hash. It freezes
the accepted sound and permits it to remain a demo-ready candidate. It does not
substitute for the remaining exact interactive TUI/device-path gesture,
restart, recall, and trigger verification; that product-path boundary must be
closed without changing the accepted render unless a correctness defect is
found.

## Interactive A/B Timing Rejection

Two subsequent real PipeWire/TUI takes used the correct source, preset, command
order, and Riotbox-only monitor path. The reviewer considered each take
acceptable in isolation but heard no meaningful difference in a direct
A-then-B comparison. Technical analysis confirms that this was not useful sound
variation: the captured source window and MC-202 selection were identical,
both renders measured `-18.6 LUFS` and `-0.3 dBTP`, and their band deltas stayed
below `0.03 dB`.

More importantly, both takes missed the declared arrangement timing in the
same way. Observer truth for each was:

- hook: `8` beats
- slam: `9` beats instead of `8`
- Fill: `8` beats instead of `4`
- Scene: `4` beats
- return before stop: `5.764713` beats instead of `8`

The cause was wall-clock input scheduling after a quantized boundary, not an
accepted-sound defect. The local observer streams are
`artifacts/audio_qa/local/user-session/RIOTBOX-1402-exact-live-a/events.ndjson`
and `RIOTBOX-1402-exact-live-b/events.ndjson`; both are correctly rejected by
`scripts/validate_feral_break_live_review_timing.py`.

These takes therefore do not close the exact interactive musical gate and do
not weaken or replace the callback-render `keep` verdict above. Future live
review takes must validate the landed `8 -> 8 -> 4 -> 4 -> 8` observer arc and
explicit stop before playback readiness is requested.

## Exact Live Timing Closeout

The first final real PipeWire/TUI timing run landed committed beats
`22 -> 30 -> 38 -> 42 -> 46` and stopped at transport position
`53.823536`. The dedicated validator accepts this as
`8 -> 8 -> 4 -> 4 -> 7.823536`, inside the declared quarter-beat stop
tolerance. Its observer stream is
`artifacts/audio_qa/local/user-session/RIOTBOX-1402-exact-live-timing-v4/events.ndjson`
and validates independently as `riotbox.user_session_observer.v1`.

The concurrent sink-monitor WAV from that run is excluded from listening
evidence: its raw capture measured approximately `-3.90 dBFS RMS`, reached
digital full scale, and reported an invalid positive true peak. The timing
evidence remains valid, but the contaminated WAV was never presented as the
closeout candidate.

## Exact Live Human Pass

A fresh real TUI/audio-callback run routed only the Riotbox sink input into a
temporary PipeWire null sink while the normal desktop/Chrome output remained
on the physical device. This removed the earlier capture contamination without
changing the Riotbox runtime path. The bounded candidate was trimmed from the
silent capture pre-roll with a short capture-edge fade; no gain, compression,
or musical processing was added.

The accepted live take has these identities and technical properties:

- observer:
  `artifacts/audio_qa/local/user-session/RIOTBOX-1402-exact-live-isolated-v1/events.ndjson`
- callback capture: `/tmp/RIOTBOX-1402-exact-live-isolated-candidate.wav`
- SHA-256: `327f9d4d00bd18c294bcf26f86c8b8a3b23f8e4f85474572735139d627d5ce61`
- format: 48 kHz, stereo, 24-bit PCM presentation file
- duration: `14.787` seconds
- integrated loudness: `-18.6 LUFS`
- true peak: `-0.3 dBTP`; no clipping
- typed bass owner: `unassigned`; bass pressure was not a review target

The observer landed `w -> s -> f -> y -> Y+D` at transport positions
`50.067233 -> 58.058831 -> 66.100849 -> 70.084042 -> 74.142866` and stopped
at `81.907573`. The validator accepts the take as
`8 -> 8 -> 4 -> 4 -> 7.90757`, again inside the declared stop tolerance.
Time-local analysis also distinguishes the intended arrangement changes: the
Fill is about `2.25 dB` louder in full-band RMS than the preceding slam and has
substantially more high-band energy, the following scene section drops about
`4.55 dB`, and the return recovers about `3.25 dB` while remaining below the
Fill peak. The review therefore did not ask the listener to distinguish
mechanically identical renders.

After the exact artifact was announced and the reviewer explicitly confirmed
fresh readiness, the complete candidate was played once. The resulting verdict
was `keep`: the individual elements were musically strong and usable, although
the reviewer would likely arrange the loop differently in later performance
work.

This records `human_verdict: keep` for the exact live artifact. The candidate
passes as usable musical material with a clear early hook, transformed-but-
present source character, audible impact and contrast, and live replay value.
The structured strongest-element field remains `silence`, based on Markus's
earlier explicit praise for the breaks and pauses in the same frozen sound
recipe; the hardest active sound layer is the TR-909 drum/transient handoff.
These are separate claims: musical negative space can be the strongest feature
while the drums provide the physical hit.
The limitation is equally important: the reviewed eight-bar action arc is QA
choreography, not a preferred fixed composition or default loop. The Alpha UX
should preserve the passed elements while allowing the performer to confirm,
trigger, and loop them in a different order. The local structured review is
`artifacts/audio_qa/local/listening-reviews/RIOTBOX-1402-exact-live-327f9d4d/review.json`;
it is intentionally ignored as local audio-QA state, while this review keeps
the durable hash-bound verdict and product consequence in repository history.
