# RIOTBOX-1579 — one isolated generated loaded-recorder observation

Date: 2026-10-16. Classification: maintenance/regression on the accepted P023
recording path, RIOTBOX-1041/P017. **FAILED; invocation consumed; no retry.**
The 250-ms preflight passed; the first two-bar case was interrupted by an
observer lifetime check after its stopped-window milestone. Eight- and
sixteen-bar cases did not start. No finished recording qualification exists.

## Authorization and exact execution identity

The user explicitly replied “ja bitte teste” on 2026-10-16 to the proposed
once-only isolated generated test. The operational approval record names the
exact scope: one existing 250-ms startup check and at most one existing 2/8/16-bar
recording at 132 BPM, first failure stops. No physical output, DAW, real source/
capture, Holdout, commercial audio or human playback. This is not a listening
review or a new musical mechanism.

Only the sole reviewed executable fence in
[phase V1](../benchmarks/loaded_recorder_host_riotbox_1579_v1.md) was invoked.
It used clean reviewed revision `024274a2c34d6e3b284709d373cabf3b97c73db7`,
full local source-free CI (`/tmp/riotbox-1579-full-ci.log`) and successful native
[Rust CI run 38708784414, attempt 1](https://github.com/marang/riotbox/actions/runs/38708784414)
at that exact revision. Preparation [PR #1682](https://github.com/marang/riotbox/pull/1682)
had merged as `996795956d693b6ff69a4f536b71d52d67cd1a96`; the two trees were
verified identical. The result branch was initially based on the actual reviewed
head, then moved to that in-content-identical main only after execution. No build
or tracked edit occurred between prospective binding and invocation.

Bound default-feature binaries (SHA256):

| Binary | Digest |
| --- | --- |
| `cpal_spike` | `4b50d6b5d491e60cadc2fbb4f39d42e212c52a2e6793e453c5a0a4d3d5939d5a` |
| `riotbox-app` | `6973e4831253a628c92050f656e73113d0b5f2bb776fb3147facf24eb2b6c923` |
| `dense_break_live_path_render` | `af560dcb31d3956a571eeb270db4fe81a06e24601847b354b8ee4246eb56adf2` |
| `libriotbox_silent_alsa_diag.so` | `335f6d415b05b43749b35f7808660d56e46c63ce9322aae4c02ee4038ff71217` |

The fence verified phase/loaded V1/fixture V1/containment V3/diagnostic V4
contract hashes and the exact native CI API response before host access. No
duration, rate, buffer, priority, DSP, algorithm, threshold or contract changed.

## Owned evidence

All raw evidence remains local and ignored under the one frozen owner:
`artifacts/audio_qa/local-loaded-recorder-riotbox-1579-v1-2026-10-15`.
The owner date is prospective, not the execution date. No alternate owner was
created and no old ignored attempt was opened. Paths below are relative to it;
digests are SHA256 of the original bytes, not JSON reserialization.

| Record | Digest |
| --- | --- |
| `fixture-v1/preparation.json` | `29667b13bb851036dd0e0513daf25fb98325979dafb72042a48d6bd38ff5c321` |
| `fresh-authorization.json` | `1a663df0bc4a03e115ed080d82c9c91b44e05e4a9c13ac9d43114ea6920577dd` |
| `prospective-binding.json` | `ae8c0aebf070b65c88cd3dfa163da0ca07debf5b619c0485dc9118b6dacae500` |
| `invocation-consumed.json` | `d41e53adf8fec5ddeed19b595065321c76ed06d6ec47153c54f468118f868c37` |
| `attempt-01/result.json` | `32696cd3624238e97783a24b9e7134ff664e44b67778306466d5353d686f3d6b` |
| `attempt-01/run-1.routes.ndjson.gz` | `e478269996af6d6c17d6ddee2cced6b3ee35759cb2f316d89ac6eaf7574faacb` |
| `attempt-01/observer-2bars.ndjson` | `24969e6c68b0e65ffc520bcc0ba4b7d888a1409bee8051cdf2dcc78acd0eef14` |
| `postflight-verification.json` | `c71aae27db9bff20652146bb4322eb28fce9a825e952f75935d84baee9d3aca1` |

The exclusive consumed marker was written before entering the operator. It binds
the approval, prospective binding, execution revision and native CI run/attempt/
response digest. Invocation log: `/tmp/riotbox-1579-actual-host-v1.log` (exit 1).
Generated baseline admission/identities were rechecked before and after the
attempt and independently during postflight; they remain unchanged. Mutable
run-Session siblings were allocated for all three declared cases; allocation
does not mean all cases ran. The five named composite contributors are TR-909,
MC-202, W-30 preview, W-30 resample tap and Source Monitor. This names the composite,
not independent activity or qualification of each lane.

## Observed sequence and first failure

Real non-root local Linux user session, UID 1000, active/nonremote, PulseAudio
on PipeWire 1.6.9; this was not a sandbox-only audio result. The operator used
its private PipeWire-only no-fallback ALSA child environment and one isolated
stereo null sink. Preflight PID 2453145: 23 callbacks, zero reported stream errors
and scratch overflows; diagnostic geometry 44,100 Hz/stereo, 1024 buffer frames,
512 period frames, 23 avail/write calls each, zero negative returns/EPIPE.

The first recorder child was PID 2453489. The retained route log establishes:

| Evidence | Linux CLOCK_MONOTONIC ns | Observation |
| --- | --- | --- |
| Route record 5 | query `[324031739050026, 324031759171317]` | Client 88; audio Node 86/serial 4140; stereo links 44/91 |
| Stopped-window marker | complete `324037146525386`; stopped `324037155247049` | two bars/eight beats, 160364 frames at 44,100 Hz/stereo; complete/stopped true, fault count 0 |
| Route record 40 | query `[324037180256983, 324037198420307]` | original Node absent; query begins after completion and stop |
| Route record 41 | query `[324037335316216, 324037352857500]` | global ID 86 now belongs to a `PipeWire:Interface:Client`, `pw-dump`, serial 4347; no recorder route |

At record 41, `scripts/loaded_recorder_process.py` rejected any object occupying
the admitted global ID when `node_present()` returned false, producing
`EvidenceError: admitted loaded node identity changed`. The shared typed lifetime
helper treats a valid different interface as original-Node absence, but the
caller's additional check treats it as replacement. This directly identifies a
terminal observer-ID-recycling false positive. It is **not evidence of an unsafe
audio reroute or of the original ALSA failure's cause**. The frozen attempt's
result nevertheless remains failed; no gate is relaxed or pass reconstructed.

The retained observer contains only the stopped-window milestone. Recorder stdout
and stderr are empty. No final ready JSON, published master WAV/proof, new receipt
or post-recording Core readiness result exists. A stop marker is not a recording
success event, and no WAV sample metrics are invented. `runs=[]` means no case
completed qualification, not that the recorder never started.

Run-1 diagnostics retain 470 avail/write calls each, zero negative returns/EPIPE,
one prepare, zero recover, and a closed measured playback PCM with the same
44,100 Hz/stereo, 1024/512-frame geometry. These are instrumented partial-run
measurements, not a completed recording/endurance or zero-observer-cost pass.

## Cleanup, review and next boundary

The supervised operator verified group and attributed-stream cleanup and removed
the exact owned sink/module; `cleanup_verified=true`, no cleanup error/notes.
A separate read-only postflight then re-admitted unchanged generated inputs,
verified both recorded child PIDs absent, no attributed streams, no owned sink
Node/module and exact equality of original default/mute/volume (69% each channel,
unmuted). No physical/default output setting was changed. The independent check
does not reconstruct historical process-group membership; its bounds are explicit
in `postflight-verification.json`. No second host/runtime invocation occurred.

Independent Spec/Evidence review traced the raw route/observer/diagnostic records,
approval/consumption binding and declared generated files. The root separately
verified the decisive route transition and current cleanup. The demonstrated
observer defect is assigned to
[RIOTBOX-1580](https://linear.app/riotbox/issue/RIOTBOX-1580/correct-terminal-non-node-id-recycling-in-loaded-recorder-observation):
a deterministic generated regression and a narrowly versioned source-free repair,
not a change inside this consumed phase. V1 contracts/evidence remain frozen.

RIOTBOX-1579 closes only the once-only observation and honest negative report.
RIOTBOX-1041/1036 stay open. No musical/human/hardness/source-general, physical
device, latency, endurance or release claim is added. No source search, real
source/capture, Holdout/commercial audio, DAW or human playback took place. A
future actual observation still needs a new prospectively reviewed phase with
exact bindings and fresh explicit approval; RIOTBOX-1580 does not grant one.

Branch review under Maintainer/Product/Risk lenses found no defect in the
outcome-report/environment-note diff. The existing P2 observer defect above is
deferred to RIOTBOX-1580, not hidden or marked fixed by this report. Root
self-review also found no additional issue: approval and execution identities,
first-failure outcome, absent recording qualification and cleanup limits agree
with the retained records. No production/Rust/module/contract change is included.
Result-branch source-free CI log: `/tmp/riotbox-1579-result-full-ci.log`; native
PR CI and normal merge/main-sync/Linear closeout remain required workflow gates.
