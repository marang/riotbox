# Final articulated live stop envelope — RIOTBOX-1604

Date: 2026-10-17. Maintenance/regression protecting the existing P023 live
output/recording path, following checkpoint1603 on main `dd89ec9`.
No new musical mechanism, source result or taste qualification.

## Result and compatibility

The existing live W-30 preview stop gain now bounds the final per-channel
sample after PitchDive history and FilterSlam processing, before adding that
lane to the mix. It is consumed once per frame, not per channel, and is not
applied a second time to other contributors. The fade duration, latch and
reset/retrigger/resume topology are unchanged.

RBX-465 freezes a private construction-time placement enum in the existing
callback state. Only production callback preparation selects `FinalOutput`.
Both public historical offline producers retain `HistoricalPreArticulation`,
their original multiplication order and exact samples, including old stopped
tails. No public plan/mixer interface, mutable setter, second renderer/clock,
Action, Session/App state, schema, dependency or frozen gesture curve changed.
Historical offline output cannot qualify corrected live-stop behavior.

Musician action: Pause/Stop of already-playing articulated W-30 material.
Expected result: final lane output reaches silence within the existing5ms
callback fade rather than emitting old history for the rest of a longer block.
Transport acknowledgment, material identity and existing display semantics stay
unchanged. This is exact generated callback-output proof, not measured physical
latency, a human listening pass or a new source-derived musical claim.

## Genuine RED to GREEN

The owning-module test
`runtime::shared_transport_tr909::output_callback::tests::w30_articulated_stop::active_pitch_dive_is_silent_after_five_ms_inside_the_first_stopped_block`
uses the exact prepared production callback, generated looping PCM at1000Hz,
active PitchDive at beat9.25, then50 stopped frames. Its final tail assertion
actually failed before production edits and passed after the placement repair.
Both runs execute exactly1 test. The preceding short `--exact` filter selected0
and was corrected; that non-execution is not RED evidence.

- RED `/tmp/riotbox-1604-pitch-stop-red.log`, SHA-256
  `240078cecbdb61593713846a9dcc79d01b9b62ec2979e83547fe6523d78cbc1f`.
- GREEN `/tmp/riotbox-1604-pitch-stop-green.log`, SHA-256
  `646ca49d2b017b9310127d465805c89ac04dec965e8ab96d6ddcf8f809b3fbf4`.

## Durable controls and actual execution

The new callback test owner contains7 executed passing tests, including:

- Dry/PitchDive and FilterSlam Close/Hold/Return, mono/stereo at48kHz; the last
  fade frame remains active on each channel and every later sample is zero.
- Historical public-offline FilterSlam counterfactuals actually produce nonzero
  tails in all6 corresponding cases. Those fixtures therefore excite the bug
  rather than checking already-silent filter state.
- Whole2400-frame versus239+1+2160 stopped partitions, exact concatenated PCM,
  repeated latched silence, stationary timing and unchanged material.
- Resume before fade completion, after completion without a later stopped
  callback, and after a later latched callback. Existing activation/history
  topology is tested separately, not replaced with a false broad pause/resume
  partition-invariance promise.
- Explicit stopped retrigger publishes realistic replacement material, clearing
  superseded articulation under RBX-306.
- Preview plus one named resample-tap contributor equals the exact sum of their
  isolated controls with inactive limiter. No extra preview fade reaches the tap.

All actual callback invocations use the existing finite-PCM/zero-heap wrapper,
including cold setup's first processing call. No device, audio file or output
route is constructed. Source-monitor PCM, TR-909 and MC-202 remain absent except
the explicitly named preview/tap composition.

Expanded-module log `/tmp/riotbox-1604-articulated-stop-expanded.log`,7passed,
SHA-256 `2e38b34376f0d5a75153162058cc622563896a2daa1315df0352ede9eb34266e`.
Filter counterfactual also executed alone,1passed, in
`/tmp/riotbox-1604-filter-counterfactual.log`.

Separate actual-live versus public-historical running comparison executes
Dry/PitchDive/FilterSlam, mono/stereo, through13 uninterrupted beats. Every sample
bit agrees, covering complete frozen gesture windows. Log
`/tmp/riotbox-1604-running-compat.log`,1passed.

Six historical offline digests were captured from the freshly built unmodified
dd89ec9 Audio library before normal candidate-library builds. Each covers
running→stop→resume with120/240/2400 stopped frames and with/without another
stopped block, mono/stereo. Literal hashes are committed in
`w30_stop_compat.rs`; candidate tests assert them rather than regenerating
expectations. Baseline Audio rlib SHA-256
`f325f49e968fc5a4b9d938a3af3be09d6dd9c57967c8d121c920543549c92325`.
Pre-fix capture log `/tmp/riotbox-1604-historical-pre-fix.log`, SHA-256
`61761eaa49eaf5fdcc3fe1059d4c3a4a925f5e6e23b01141fd6ba3a9cf822a43`.
Candidate `/tmp/riotbox-1604-offline-compat.log`,1test/6digest controls passed.
An initial harness compilation confused Vec samples with a report wrapper;
it was corrected before baseline execution and is not product-failure evidence.

## Review finding and disposition

[P2, fixed] Test-fixture preparation added an avoidable by-value wrapper around
large callback preparation. Originating lens: coordinator Test Evidence.
The broad Audio suite aborted in the existing three-callback PitchDive-history
control; the exact single test independently reproduced the stack overflow.
Removing only that additional wrapper, keeping format parameters on the existing
constructor, restored the normal test-thread pass. No larger thread stack,
production workaround, ignored test or weakened assertion was used.

Exact RED `/tmp/riotbox-1604-fixture-stack-red.log`, SHA-256
`41cf460172ac7ab76d26b52a52333fd039a4af16172af2c0d523f695fb9f8888`;
GREEN `/tmp/riotbox-1604-fixture-stack-green.log`, SHA-256
`0083bf1847e4b9c9097d08775df5326379d4f4964a4b6bf189f9749b3f777b4f`.
The subsequent broad `cargo test -p riotbox-audio --lib` executed378passed,
0failed,1existing ignored manual release-mode snapshot benchmark. Log
`/tmp/riotbox-1604-audio-lib-green.log`, SHA-256
`6d0e6dc3a67228463ab4dfc0448a222224f329bbca562ab1f02012d4a5b98d63`.

Independent Spec And Evidence Auditor and Maintainer/Adversarial/Rust/Realtime
reviews retained0 production findings. They read all constructors/consumers and
new tests without running Cargo; coordinator owns execution evidence. Final
test-fixture follow-up review, self-review, strict Clippy and full exact-head
local/native CI remain recorded separately in PR/Linear before merge.
Existing large renderer/state owners were reviewed as cohesive legacy; this
small policy repair does not justify a mechanical file split or mixer rewrite.

## Bounds and remaining work

No actual user Session migration, source/capture/Holdout/commercial/ignored
material access or directory discovery, host/device/service/TTY runtime or
observation, playback, DAW or Windows/TUI expansion. No human/musical/hardness/
source-general/physical-latency/endurance/release pass. Frozen qualification
manifests and historical evidence remain untouched.1589 remains passed/consumed;
1041/1036 and original ALSA cause remain open.

1605preset/Undo,1606TR-909 overflow phase and1607snapshot future provenance remain
separate unfixed checkpoint findings. This repair does not silently close them.
