# RIOTBOX-1447 W-30 Gesture Re-entry Qualification

Date: 2026-08-21
Partition: Development only
Decisions: RBX-306, RBX-307
Result: qualified ordinary re-entry; gesture-combination taste not assessed

## Scope

The three performer-owned W-30 gestures were already individually qualified:
Hook Turnaround as phrase variation, Pitch Dive as destructive exit, and Filter
Slam as a long build/return. RIOTBOX-1447 did not requalify their DSP or ask
whether a particular combination forms a superior arrangement.

Source-blind state-flow inspection instead found an integration defect: the
durable `pitch_dive_v1` articulation remains intentionally silent after its
terminal exit, but an ordinary W-30 hit or recall did not supersede that state.
The normal performer path could therefore remain silent indefinitely. RBX-306
now makes committed live recall, trigger, raw/promoted audition, and damage
profile actions clear a preceding timed articulation identically in live
execution and replay. Gesture actions still replace the typed profile; no DSP,
timeline, threshold, or automatic expiry changed.

## Frozen Qualification And Access

The bounded journey contract is
`docs/benchmarks/w30_gesture_vocabulary_golden_path_qualification_v1.json`,
SHA-256
`80b3433c20168e8b1cc42c399af017faa21a06e509b8240e569485a001d3a7fb`.
It allowed one exact registered Development source and no directory discovery,
Holdout, commercial-reference, replacement-source, retuning, or gesture
stacking.

The accepted technical session is `2026-08-21-c`. Its local access log is
`artifacts/development/riotbox-1447/access-log-2026-08-21-c.json`, SHA-256
`78a71454981adfbc056749a44c99148d7604c05b1d222815f722f21d6cb686e3`.
The separate review access log is
`artifacts/development/riotbox-1447/review-access-log-2026-08-21-c.json`,
SHA-256
`a9e9bf071c00d9c3b829630f1a43b130341ef15b1fa161dc9076a0eedfee6edc`.
Both verified the source hash before and after access and record no directory,
Holdout, or commercial-reference access.

An earlier local session `2026-08-21-b` was invalidated fail-closed before
playback because its continuous-journey writer retained only the final render
step. Exact-artifact preflight caught the 6.53-second duration mismatch. The
writer and validator were corrected to require all six render steps and the
exact expected frame count; no verdict was transferred from the rejected
artifact.

## Technical Result

The exact committed suffix was Hook Turnaround, ordinary trigger, Pitch Dive,
ordinary trigger, Filter Slam, ordinary trigger. Each unchanged gesture passed
its existing mechanism gate. Hook Turnaround produced reverse/choke delta RMS
of `0.067278` / `0.048264`, Pitch Dive produced final-window delta RMS
`0.088090`, and Filter Slam produced effect-through-return delta RMS
`0.021836`. Missing-source controls remained silent and every control and
candidate remained clip- and limiter-free.

All three ordinary re-entry states cleared the preceding articulation and
rendered active W-30 output with RMS `0.067734`. Session JSON round-trip and the
complete six-action replay suffix matched the committed W-30 state. The
continuous 37-beat RuntimeMix render was sample-exact across 128- and 257-frame
callback partitions. This proves transition integrity, not a preferred musical
ordering or automatic arrangement policy.

## Human Review

After technical qualification, one source-first artifact presented 3.684
seconds of registered Development source, one second of silence, and the exact
continuous isolated W-30 render. The stereo 48 kHz PCM16 artifact is
`21.723812` seconds, measures `-22.2` LUFS with `-4.8` dBFS true peak, contains
no NaN/Inf or clipping, and has SHA-256
`f47fd90327295afda645626fb1a929df1caaa87ede79ab1e9aace48a3199b5eb`.
Its W-30 tail is sample-exact to the qualified continuous render. Playback
reached the announced endpoint and left no playback process active.

The musician found the presented transitions acceptable, then clarified that
the three effects had already been confirmed individually and that musical
effect combinations should be evaluated later only when there is a concrete
reason. The durable interpretation is therefore:

- `transition_integrity_verdict: keep`
- `ordinary_reentry_after_timed_articulation: clean`
- `combined_gesture_musical_verdict: unverified`
- `automatic_or_preferred_ordering_claim: false`

After CI completion, a focused non-combination confirmation presented the
unchanged normal control, one second of silence, the unchanged Pitch Dive
candidate, one second of silence, and the unchanged ordinary re-entry four
times. Their SHA-256 values were respectively
`7140c8f24e383dc6a7cb75bc6183e03727ef8b5f068b28e9d08ead8371a5ebab`,
`40ecd9ed895138c21954fb7164743c0b7c8bf97af3cdc4ec1c37dc689fa079bd`,
and `1d7056a220f3c0c2194f2fdb6ef9b4d3dd4d0f852a97f2045e9a5285355a0f9a`.
Playback reached the announced endpoint and left no player active. The musician
confirmed that the normal beat returned after Pitch Dive and noted that its
onset sounded slightly different. Exact follow-up analysis found the first 100
milliseconds waveform-correlated at `1.000` with the matching normal chop step,
but with a stronger explicit-retrigger attack (`0.245` versus `0.160` RMS over
the first 20 milliseconds). From 100 to 900 milliseconds the correlation was
`0.996`. The Hook Turnaround, Pitch Dive, and Filter Slam re-entry files were
byte-identical to one another. The audible difference was therefore the normal
performer-trigger attack, not residual Pitch Dive processing. This confirmation
strengthens only the ordinary-re-entry verdict; it does not assess a gesture
combination or claim that a fresh trigger is identical to an already-running
chop step.

## Claim Boundary

RIOTBOX-1447 qualifies explicit ordinary W-30 recovery after a timed gesture
and proves identical Session/replay behavior through the exact product output
path. It does not add or reapprove an effect, rank the three effects, endorse
their tested order as an arrangement, authorize automatic selection or
stacking, or claim source-general, Holdout, demo, release, or percussive-hard
readiness. A future combination review requires a concrete musical use case and
a new bounded artifact assignment, fresh readiness, and a new verdict rather
than inheriting this transition verdict.

## Subsequent Combination Review

RIOTBOX-1448 later assigned the unchanged exact journey to a new bounded
musical-review purpose with fresh preflight, readiness, playback, and a new
structured verdict. That separate review keeps the fixed sequence as coherent
and harmonically compatible while preserving the no-ranking, no-automatic-
ordering, and no-stacking boundaries. See
`docs/reviews/riotbox_1448_w30_gesture_sequence_musical_review_2026-08-21.md`.
