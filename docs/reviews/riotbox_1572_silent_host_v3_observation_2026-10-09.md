# RIOTBOX-1572 — one silent-host V3 observation

Date: 2026-10-09. P017 operational observation of the accepted P023 callback
path, not an audible slice or musical progress.

## Result and authorization

**Failed: real backend stream error in run 2; cleanup verified.** The user
explicitly authorized this new once-only silent virtual-output observation.
The [prospective contract](../benchmarks/silent_host_riotbox_1572_once_2026-10-09.md)
was frozen and independently reviewed before host access. Exactly one committed
invocation ran; no repeat, alternate owner, threshold change or post-result
reclassification. V1/V2 consumed evidence was not opened or changed.

The consumption marker's filesystem mtime is approximately
`2026-10-09T19:33:55.889Z`; result mtime is `2026-10-09T19:35:33.919Z`.
These timestamps are operational file evidence, not monotonic run duration.

## Prospective identities and gates

Clean executed revision: `a3069c7051d616de4335bbbec3bfb6f85068ee97`, based on
merged V3 repair `58a4eccddf2ff35c0d2e54a0cd578d2da44b30ff`.

- V3 protocol: `a0934097be7b2f160ff698b3d69c1dd7180e86defcdc0974c5e23606297ef85b`.
- Default-feature `cpal_spike`: `58f6103c3f34ae3837a0b059c209052c36fa5398ded027a4b959dcba67993fc4`.
- Default-feature `silent_host_probe`: `59de339f9488a93958f240c26fea6456bce9e9f1ec872696d75efcd7a7fe778f`.
- Prospective binding: `4587402ebbc94ffdb94fb7a936af1e12a4715a91e55379ea392e614847f0f5b8`.
- Consumption marker: `d04b525eed592bac560f090026bf316846b98f23d74f50b7a976e67540faf439`.
- Private PipeWire-only ALSA config: `8611a4f60d398cf3117ddccdf47255bb4e100ac701ce6a47ca7f9e99618af7f5`.

Prospective and admission revision/protocol/binary identities match. Final
default-feature build followed native CI; no build or tracked change intervened
between binding and invocation.

All 140 generated host fixtures and full local source-free `just ci` passed
(`/tmp/riotbox-1572-ci-pre.log`). Six new tests execute the exact phase fence
with generated Git/operator adapters, checking mismatches, once-only use and
early-failure consumption. Two independent Spec/Evidence and Adversarial
reviews and self-review retained zero P0–P3 findings; additional generated
before/after-marker SIGTERM checks passed. Native
[Rust CI 37980167201](https://github.com/marang/riotbox/actions/runs/37980167201)
passed on the exact executed revision before host access.

## Actual context and observations

Non-container active local Linux user session: UID 1000, `Active=yes`,
`Remote=no`, `PulseAudio (on PipeWire 1.6.9)`. This was real-session evidence,
not a sandbox-only failure. Owned module ID `536870916`, sink Node ID `44`,
serial `1348`; private endpoint `pipewire-0` and exact route containment were
admitted. The V3 multiline-metadata blocker did not recur.

CPAL reports `Alsa` / `default`, F32 stereo at 44100 Hz, buffer range
32–2097152. Here `default` is the child-private PipeWire-only ALSA configuration,
not evidence of routing to the user's physical/default output. Range metadata
does not establish the actual callback buffer size or device latency.

- 250-ms preflight: typed `Ok`, 23 callbacks, zero stream errors/scratch
  overflows; wrapper lifecycle 0.526 seconds, one route observation.
- Run 1: complete 62-record transcript with 60 samples and terminal `Stopped`;
  callbacks advance from 0 to 5170, driver elapsed 60010 ms, zero stream errors
  and scratch overflows. Wrapper lifecycle 60.334 seconds, 432 route
  observations. Independent complete-transcript validation passes.
- Run 2: 38 records (started, 36 samples, stopped). Sample 35 is `Running/ok`
  with zero errors. Sample 36 at 36004 ms is `Faulted/failed`, callback count
  3102, stream-error count 1, scratch-overflow count 0, with retained detail
  `Buffer underrun/overrun occurred.` Terminal record at 36005 ms is `Stopped`
  and retains that error. The unchanged validator independently rejects the
  transcript for the same explicit stream error.
- Run 3: not started. The complete three-run gate therefore fails.

Final recorded maximum callback gaps are 26327 microseconds for run 1 and
25154 for run 2 (preflight 21339). These are observations only: there is no
gap threshold, xrun/latency qualification or inferred cause. Evidence proves
that the backend reported an error; it does not distinguish underrun from
overrun or establish whether host scheduling, ALSA/PipeWire buffering or the
runtime caused it. No algorithm, buffer or threshold was changed afterward.

## Cleanup and independently checked evidence

Operator result is `failed`, `cleanup_verified=true`, no cleanup error. Its
existing supervisor verifies group/stream cleanup before sink removal. Separate
read-only checks establish unchanged original default/mute/volume, absent
exact owned module and named Node, and absent preflight/run-1/run-2 PIDs.
No unrelated resource was unloaded, moved, restored or changed. Direct PID
absence is supplemental evidence, not a new adversarial PID-reuse guarantee.

Preflight and completed run-1 stdout/route digests match their operator records;
run-1 complete transcript passes the independent validator. Run-2 terminal
stop and rejection are retained separately; incomplete qualification is never
promoted to a pass.

Exact ignored phase:
`artifacts/audio_qa/local-silent-host-riotbox-1572-v3-2026-10-09`, owner `attempt-01`.
Bulky metadata, routing logs and transcripts stay local/ignored.

- Admission: `0975ea704a31e7bbc177f3034b6ef843cb808d57a4d88239ed1c3147fea216be`.
- Result: `9f130973b65ef5e299687bba3bccd03149879ecd9dd61b45684be1f419631b41`.
- Independent postchecks: `03133c46561b3858f939bfa9eb12ab360fe4e9d6cda5cbcb93461227e7d11bdc`.
- Failed-run checks: `ef32e950ce6b8adaa9c038bf6387de1e1c95585d5d8ba4285667a1ae85ba4b1b`.
- Failed run-2 stdout: `ce5934c1284fb71ff875122926bf1c5398e0527366b45f36495a5b5c8d8008cf`.

Execution/postcheck logs: `/tmp/riotbox-1572-host-once.log` and
`/tmp/riotbox-1572-postchecks.log`. Report review, final native PR CI and normal
merge/main/branch closeout are separate remaining gates at report preparation;
their actual completion is to be recorded in Linear and the project journal.

## Disposition and limits

This completes the one authorized attempt as bounded negative runtime evidence,
not the three-run callback gate. RIOTBOX-1041 stays open.
[RIOTBOX-1573](https://linear.app/riotbox/issue/RIOTBOX-1573/diagnose-recorded-silent-v3-alsapipewire-buffer-error-without)
owns source-free diagnosis of the recorded error as the next safe step;
no repair-and-retry is authorized
by this consumed phase. No physical-device, loaded-runtime, latency/xrun,
endurance, source, musical, hardness or release pass. No source/Holdout/
commercial/capture audio, source-directory discovery, DAW, physical output,
TUI or human playback occurred. Frozen V3, operator and Rust driver are unchanged.
