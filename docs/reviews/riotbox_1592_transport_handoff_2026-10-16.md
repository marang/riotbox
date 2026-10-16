# RIOTBOX-1592 — observed progress and explicit transport intent

Date: 2026-10-16. Maintenance protecting P023 performer navigation and recording
under RIOTBOX-1041. Parent checkpoint: RIOTBOX-1591 / PR #1696, integrated as
`eac214b3136807c725016c26fd4034944307bd9c`. Contract: RBX-453 and
[Audio Core §7.1.2](../specs/audio_core_spec.md#712-explicit-position-intent-and-callback-acknowledgment).

## Reproduced defects

Two deterministic pre-fix failures demonstrate opposite directions of the same
handoff problem, not frequency or the cause of any retained host incident:

- Exact production output callback: observe beat 0.2, let ten healthy 10-ms
  callbacks advance to 0.4, then echo the old observation. The next completion
  is 0.22 instead of 0.42, and generated ramp PCM repeats the earlier excerpt.
  RED: `/tmp/riotbox-1592-callback-feedback-red.log`.
- Actual App/Core navigation: commit PreviousBar from 12 to 8, then consume an
  old same-running 12.5 observation before delivering that seek. Session/runtime
  become 12.5 instead of remaining 8.
  RED: `/tmp/riotbox-1592-committed-seek-red.log`.

Both regressions pass after repair. The first executes the processor installed
in CPAL's callback without constructing a device; the second executes the
existing source-map action/commit path with an in-memory metadata graph.

## Implementation and ownership

Live control distinguishes typed Preserve from Reposition. Checked runtime-only
request identities make repeated numeric targets distinguishable. The callback
applies an observed identity at most once and publishes its acknowledgment with
completed timing. The public timing value retains its three fields; its existing
optional getter withholds old acknowledgments. Latest-state publication can
supersede an unobserved request; this is not another command queue.

The existing App driver retains only an undelivered committed target. One private
CLI transport handoff dispatches it before timing consumption and also dispatches
transport intent committed by a crossed boundary. Interactive and recorder paths
share that order. Core/Session remains the position/action/replay authority.
Invalid/exhausted requests fail before mutation; device shutdown and recorder
abort/no-publication remain unconditional on failure.

Explicit live seeks also invalidate the drum renderer's old position tolerance:
a sub-step repeated seek must affect actual Fill PCM, not just telemetry. Musical
recipes and lane DSP are unchanged. Legacy numeric offline plans keep their old
interpretation in one ingress adapter, using the same clock and mixer. Their
public plan fields, threshold, empty-step and stopped/resumed behavior remain.

## Generated verification

- Actual callback/public getter controls cover request supersession, repeated
  same-position, tiny, forward, backward, stopped and zero seeks, paused/running
  restored startup, Pause/Play and tempo refresh. Callback heap assertions remain.
  PCM consequences are directly checked by the delayed-ramp and repeated
  sub-step Fill tests; the other matrix cases assert timing and acknowledgment.
- Common CLI handoff: six tests cover ordinary Preserve, delivery before stale
  observation, failed-delivery nonmutation, repeated zero/Stop, boundary-committed
  seek with deferred queue work, and same-runtime fractional restore.
  `/tmp/riotbox-1592-common-sync.log`.
- App library: 889 passed, one existing informational benchmark ignored in the
  complete CI's App test target. Existing recording/identity/sequence/restore
  controls remain present; no tests were disabled.
- Exact-source Loom: two callback publication tests and eight control tests pass,
  including whole-publication nonmutation for invalid/exhausted requests and
  Preserve at exhaustion. `/tmp/riotbox-1592-models.log`. Only atomic primitives
  and the test model context differ; the production owners are included directly.
- Independent read-only Rust/adversarial and spec/evidence reviews found no
  retained P0–P3 defects. Coordinator additionally reviewed product scope,
  ownership, compatibility and risk. Full local/native CI remain mandatory PR
  gates; their exact run/head/results belong to PR and Linear closeout.

The public offline compatibility witness used generated PCM only, 44.1/48 kHz,
1/2/4 channels, four callback partitions, empty steps, stopped/running/zero-tempo
states, small moves, large jumps and changing drum modes. The retained pre-fix
default-feature Audio library SHA-256 was
`881f9c555bc15d004a0dcdd02639a90aeaeece40148d83a0ba25fbd58acadf32`.
It and the candidate were separately linked with the matching locked Core
dependency. An initial mismatched Core artifact was rejected by rustc, not used
as evidence. Both successful executions report:

```text
cases=384 steps=12288 samples=4699968 sample_bits_fnv64=d020707302ed6ecf
```

Witness source `/tmp/riotbox-1592-offline-parity.rs`, SHA-256
`a8e3507cd896a82e9500540047bbde2dd3d1226980ed86fb67a076dadfc1c5a2`.
Output logs `/tmp/riotbox-1592-offline-{baseline,candidate}.log` compare equal;
each log SHA-256 is
`de2cb3077001dbd766013236edd0729a3637b0604f147af4ca022bbc8af39a58`.
This is bounded deterministic sample-bit fingerprint evidence, not an exhaustive
equivalence proof or a cryptographic hash of all PCM bytes.

## Limits

No actual host/service/device observation or audio runtime, source/capture/
Holdout/commercial access, ignored attempt, DAW or human playback. Generated
maintenance evidence grants no musical, human, hardness, source-general,
physical latency, endurance or release verdict. Consumed RIOTBOX-1589 remains
unchanged. Durable backward-navigation replay chronology is a separate known
finding owned by RIOTBOX-1593; this repair does not close it. RIOTBOX-1041/1036
and the original ALSA cause remain open.
