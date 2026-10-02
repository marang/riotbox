# RIOTBOX-1553 — bounded eight-bar real-host recording

Status: one exact eight-bar take technically accepted; bounded human recording-
continuity qualification complete. Publication closeout is tracked in the PR
and Linear issue.

## Purpose and scope

Follow RIOTBOX-1551/1552 with one V3 eight-bar recording through the existing
CPAL post-limiter callback. The selected held tonal composite previously earned
the bounded RIOTBOX-1492 two-bar loop keep. This follow-up tests longer recording
continuity and usefulness; that earlier verdict does not qualify a fresh take.
RIOTBOX-1036 remains the broader recording/export owner.

The resumed continuation authorizes this bounded attempt. No algorithm, recipe,
threshold, renderer, TUI control or persistence model changes are planned.
Capture uses an isolated silent virtual host endpoint; this is not physical
device/acoustic endurance or DAW-import qualification. Human playback requires
exact-artifact preflight, independent assessment and fresh readiness.

## Frozen attempt

- Input owner:
  `artifacts/development/riotbox-1482/product-qualification-2026-08-28-final-a/tonal_rusharp_120`.
- Exact input metadata: `session.json` and `source-graph.json` in that owner.
- Development registry: `docs/benchmarks/sound_excellence_source_corpus_v1.json`,
  case `tonal_rusharp_120`, original locator
  `data/test_audio/examples/DH_RushArp_120_A.wav`. The original is never opened.
- Selected existing owner-copy: `registered-source-owner-copy.wav`, SHA-256
  `ec2a0c930eb338bf81cd5cb4b5fef487e07c140ad40181e1d92b2a0990334e0e`.
  One logged identity read and one traced normal product hydration read only;
  preserve its existing locator in the copied Source Graph.
- Selected capture: `captures/cap-01.wav`, SHA-256
  `8460fd78a74fb1290555413ffd8160262ce88077c55fbdf09e417233d8d1f183`.
  One logged identity/copy read, one traced explicit adoption read on the fresh
  copy, and one traced normal product hydration read from that copy only.
- Protected partition comparison uses only the case/path/hash metadata in
  `docs/benchmarks/source_holdout_rotation_v3.json`; no protected audio access
  or source-directory enumeration.
- Fresh owner: `artifacts/development/riotbox-1553/take-01`. Its access log,
  route snapshots, migration trace, capture trace and product observer retain
  the actual operations. Preserve prior inputs and all failed evidence.
- Copy metadata exactly. The existing `capture_identity_migrate` tool adds
  `adopted_legacy_v1` only to copied `cap-01`; its current bytes must match the
  documented hash. This creates today's baseline, not historical authenticity.
  Explicit copied Graph override keeps publication out of the original owner.
- Recording: exactly one eight-bar V3 take, 120 BPM / 4/4, 32 beats, expected
  705,600 stereo float32 frames at 44.1 kHz / 16.000 seconds. The current Scene
  stays held; no performer gesture or musical arrangement is added.
- Contributors expected from metadata: W-30 `live_recall` of `cap-01`, existing
  TR-909 `break_reinforce`, idle MC-202, `riotbox_only` monitor, idle resample tap.
  Recheck actual restored observer state before assigning the resulting master.
- Verify a source-free silent CPAL probe first. Bind the actual process PID to
  PipeWire Client and Node, require the exact isolated sink target and active
  stereo links, no-fallback/no-reconnect/no-move properties, and no outgoing
  sink link. Keep unmatched snapshots. Never alter default sink or volumes.
- Failed identity, route, timing, health, artifact or lineage gates stop the
  attempt. No silent substitution, second take or audio repair is authorized.

### Source-free preflight corrections

The first silent CPAL probe negotiated 44.1 kHz using the production default
device selection. The initial proposed 48 kHz geometry is retained separately
in local `attempt-plan-48000-before-probe.json`; before any selected audio read,
the active attempt records the observed 44.1 kHz / 705,600-frame geometry.
This is an operational device expectation, not algorithm or threshold tuning.

Independent Spec/Evidence and Adversarial review also found that a blocked
`pw-dump`/`pactl` call or supervisor interruption could evade the operator's
polling deadline. Metadata commands now have explicit timeouts, the host process
has a separate hard watchdog, and supervision failures/interruption terminate
its entire owned process group. A second source-free probe and route mutation
checks verify the corrected operator before admission. Neither finding changes
product code; no source or capture had been opened when they were resolved.

The first capture invocation was rejected before process launch because the
previous HDMI default sink had disappeared; the current physical default is
the analog sink. No take or additional source/capture hydration occurred.
Retain `take-prelaunch-blocked.json` and the original host baseline. One fresh
source-free preflight binds the unchanged isolated sink to the current host
baseline, preserving its default output. This does not grant a second take:
the budget remains one actual capture and the input-read quotas stay unchanged.

## Required result

Verify the exact WAV, proof, receipt, committed Action, observer and saved
Session; retain actual callback endpoints/faults and full V3 readiness. Analyze
format, frames, duration, peak/RMS/LUFS/true peak, silence/clips, local continuity
and raw boundaries without modifying the take. A metadata-only restore/report
may check saved contracts without reopening source or capture audio.

Only after technical acceptance, prepare one artifact-bound independent
pre-listen assessment and a factual 16-second review brief. Review sustained
loop continuity and recording usefulness. Bound playback, verify stream exit
and silence, then record the listener's actual verdict. No hardness,
source-general, demo-bank, release or P023 completion claim follows.

## Exact take and technical result

The one actual take captured beat 4 through beat 36 at 120 BPM. Its raw callback
endpoints are `4.000000000000004` and `35.99999999999898`, retained without
replacement by idealized positions. The stereo float32 WAV contains exactly
705,600 frames at 44.1 kHz / 16.000 seconds. Action 8 committed
`export-receipt-a-0008` with explicit `eight_bars` and V3 identity; the saved
Core Session reloaded through the read-only report and full recording receipt
readiness remains true. This report is not a second App/audio hydration pass.

- WAV SHA-256:
  `520461a582e0a12d4da79bfbb6973ab4f3bda084bd7f21ae561658eaea0919ab`.
- Proof SHA-256:
  `e43af263f81aef4c4eaf7500594c6ad5c8cdc92ed86f812aaaa43e09dead211c`.
- Sample payload SHA-256:
  `3fb289e2eb5ca46cd472ad868ae70df525bd9c0ba2d58a231e1202cdaa1f7840`.
- Preflight SHA-256:
  `3324c92efcb47b74744ade12adaacc1688ab7a1c34c8413f1c814471b33cf5c3`.
- Completed input access log SHA-256:
  `dbe8ee863d0038ac40f0547f73228c3dd3923cf95585e9a7b325c3552bea9db1`.
- Actual-route snapshots SHA-256:
  `30ac6d48cfa0cdf1a503800bc7b81b5526e4ca523ff90b6e37208327f2234e67`.
- Product observer SHA-256:
  `f6d713b4535ecc81a42f2b563b38bb161ee84eba2c058394bc9977050372c275`.

There are 1,379 captured callbacks and zero gap-over-threshold, scratch-overflow,
stream-error, transport-, tempo- or timing-window-mismatch counts. Maximum
reported callback gap is 62,426 microseconds; this is retained health telemetry,
not a claim of an underrun-free physical output device. The process-owned CPAL
stream appears in 141 valid route snapshots, both stereo links target the exact
isolated sink, and process/stream exit is verified. The temporary sink is removed
and the current physical default output remains unchanged.

File traces show exactly one normal owner-copy hydration read and one copied
capture hydration read, in addition to their admission reads and the selected
capture's adoption read. Totals are source 2 / capture 3; no original source,
other capture, Holdout or commercial audio was opened. Original Session and
Graph byte hashes remain unchanged. Source directories were not enumerated.

Exact-output preflight reports sample peak `-7.590 dBFS`, independent f64 RMS
`-19.479 dBFS`, integrated loudness `-16.9 LUFS`, estimated true peak `-5.3 dBTP`,
zero clipped/nonfinite samples and no silent frames in any of the eight bars.
Channels are sample-identical; per-channel DC is approximately `-0.003019`.
No gain, fade, crossfade, resampling or source-recipe adjustment is applied.

The initial operator incorrectly required its independent f64 RMS to equal the
stored f32-accumulation result. Reproducing existing `signal_metrics` arithmetic
matches the proof exactly: `106158` amplitude-micros, while f64 gives `106179`.
Both measurements are retained; no product algorithm or tolerance changed.

Raw end-to-start step is `0.154188` per channel, compared with internal adjacent
difference p99 `0.085891` and maximum `0.472747`. The first internal bar-boundary
step is `0.399251`, the remaining six approximately `0.000961`. These are
descriptive observations, not an inaudible-seam verdict. One ordinary 16-second
playback tests internal sustained continuity; it does not audition a repeated
end-to-start join or grant seamless-loop qualification.

Actual observer assignment confirms the expected composite: W-30 live recall of
`cap-01` plus existing TR-909 `break_reinforce` renderer support; MC-202 and the
resample tap idle, raw Source Monitor excluded by `riotbox_only`. The existing
external export actions remain non-replayable; replay does not capture again.

One inherited descriptive alignment-gate summary still says "two-bar". Its typed
Action/duration, 32-beat proof, WAV, receipt and readiness all establish eight
bars. Preserve the hash-bound take unchanged. RIOTBOX-1554 owns the small
source-free correction for future receipt wording; it does not alter this sound.

## Listening handoff

The local pack is
`artifacts/audio_qa/local/listening-reviews/RIOTBOX-1553/review.json`, with
`human_verdict: keep` limited to recording continuity. Its only candidate is the exact unprocessed WAV
above. The independent artifact-bound pre-listen assessment is complete with
no additional technical blocker. Its immutable local record is
`artifacts/development/riotbox-1553/take-01/agent-prelisten.md`, SHA-256
`f57d04607dadc1e813aaf435fe37e05b5146fe18677c3747c94d0b93983a21ff`.
It distinguishes measured validity from predicted musical usefulness; no
assessor playback occurred and no human verdict is inferred. Preserve the
prediction before subsequent feedback. Fresh listener readiness and the
bounded playback are documented below. The factual question is sustained continuity and
usefulness of the recorded hook-plus-beat composite over eight bars; no new
arrangement, isolated-drum, hardness or seamless end-to-start claim is requested.

The initially prepared one-shot playback supervisor bound the unchanged candidate
to the then-current analog output without altering default routing or volume. It stops at WAV
EOF, and has a separate 20-second failure watchdog. It verifies the process-owned
stereo route and playback-stream removal. An independent code-only safety review
found an interruption window during player launch; signal handling and guarded
process/watchdog initialization now cover that interval. Re-review found zero
remaining findings before human playback.

The listener's first readiness response led to a pre-launch rejection, not
audio playback: the current physical default had returned from analog to HDMI.
Retain local `playback-prelaunch-blocked-01.json`. Candidate and original
assessment hashes are unchanged, input access quotas unchanged, and no player
was launched. The current HDMI endpoint is available and unmuted, with its
existing 115% / +3.59 dB output volume; do not alter that volume or the default.
The re-briefed route uses this exact current HDMI output with the same bounded
16-second candidate, after metadata-only route assessment and fresh readiness.
The independent metadata-only assessment reports no additional blocker and is
preserved separately as `agent-prelisten-hdmi-addendum.md`, SHA-256
`bbf08e28d12680ee511faca118e1d2dba23e34e4dd7717fa85e875de9d7b0749`.
Original musical predictions remain unchanged. Normal playback-rate conversion
to the 48 kHz host endpoint is not a change to the captured WAV or an acoustic
output qualification.

After fresh explicit readiness for HDMI, the unchanged full candidate played
once for 16 seconds. The player completed with exit status 0 after 16.114
seconds including supervision; 122 snapshots verified its active stereo route
to HDMI serial 4551. The process-owned playback stream was removed and the
default sink was unchanged. No volume/routing modification or extra source/
capture read occurred. This verifies cessation of our playback, not silence
of unrelated desktop applications. Local `playback-01-result.json` has SHA-256
`22c72dceff9ffd4c3592e1386d8a9e22e87a065795ebdd96be750bb31007eb92`.
The listener recognized the unchanged musical material and, after clarification
that only the new recording's continuity was being judged, reported no audible
recording problem. Record this as a bounded `keep` for recording continuity,
not a renewed musical approval. No strongest musical element, source-recognition
or hook-quality verdict was requested or inferred (`none` / `not_applicable` /
`inconclusive` in the generic pack). The original agent predictions remain
unchanged: listener evidence does not establish their predicted boundary risks,
and this single ordinary playback does not qualify the repeated end-to-start
join. No further playback of the unchanged sound is needed. Demo readiness stays
unverified and `quality_claim: false`; no demo-bank promotion follows.
The completed local review JSON has SHA-256
`0aa9fce3510a51e8325494003aee3d794abdc8db87ac9484395b49dd1d2b82a5`.

The generic pack's `human_pass_allows_demo_ready_candidate` consequence is
permissive, not actual promotion; the explicitly bounded review scope above
does not grant broader demo, musical or release qualification.

## Verification and remaining closeout

The documentation-only branch review found no evidence overclaims; cited
metadata identities match the exact take. No product code, Action, Session
schema, DSP or source contract changed. Source-free `just ci` completed with
exit status 0; its full log is `/tmp/riotbox-1553-ci-final.log`. It covers Rust
tests, generated-audio and contract fixtures, format/JSON/whitespace checks and
strict all-target/all-feature Clippy, without reopening the admitted inputs.
Final independent documentation review and the short self-review found zero
remaining findings, including the narrow human-verdict scope and the generic
demo consequence. Normal PR/native-CI/merge/main/Linear publication closeout
is tracked in the associated PR and RIOTBOX-1553. RIOTBOX-1036 remains the
broader open recording/export owner; this report closes only the bounded
eight-bar callback and recording-continuity evidence gap.
