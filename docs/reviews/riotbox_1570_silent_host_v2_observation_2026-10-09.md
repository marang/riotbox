# RIOTBOX-1570 — one silent-host V2 observation

Date: 2026-10-09. Classification: P017 operational maintenance/observation of
the accepted P023 callback path, not audible or musical product progress.

## Result and authorization

**Failed admission; authorization consumed. No audio process started.**
The user explicitly authorized one isolated silent virtual-host V2 attempt,
with the existing 250-ms preflight and three 60-second runs, after the export
fix. This report records that one invocation, not a retry of V1. Its separately
reviewed [prospective contract](../benchmarks/silent_host_riotbox_1570_once_2026-10-09.md)
and RBX-439 remain unchanged. V2 JSON, numeric gates, disabled ordinary CLI,
Rust driver and existing operator were not modified after the result.

The exclusive consumption marker was created at approximately 18:30:48.886 UTC
(20:30:48.886 Vienna); the failed result was published at 18:30:49.113 UTC.
These are local filesystem modification timestamps, not monotonic duration
measurements. There was exactly one call to `execute_attempt`, with no retry,
alternative owner, changed threshold or repair-and-reclassification.

## Prospective identities and pre-execution gates

Executed clean reviewed revision:
`7201fe63001344dc1003df4d46f7c0d21c529991`, based on the merged export repair
`c3f28432f0abe66e5d0b690eaf0a46e85b92588a`.

- Pinned V2 protocol SHA-256:
  `9b8517f13285e36a6082fb57131602949c89758199d57ccef42a26f4a542bcea`.
- Default-feature `cpal_spike` SHA-256:
  `58f6103c3f34ae3837a0b059c209052c36fa5398ded027a4b959dcba67993fc4`.
- Default-feature `silent_host_probe` SHA-256:
  `59de339f9488a93958f240c26fea6456bce9e9f1ec872696d75efcd7a7fe778f`.
- Exclusive prospective binding SHA-256:
  `42e468b74bdb296b7303f2f1438c8abaa5a7d998d768519a22120a17b37d522b`.
- Exclusive consumption marker SHA-256:
  `e8b7bc77cc2d5935bd29cea5c24618f7001307852ef287f359a2a6ff39aa8e59`.

The admission identities equal the prospective binding; the marker hashes the
same captured binding bytes. The final default-feature build occurred after
native CI. No build or tracked change occurred between binding and invocation.

Complete local source-free `just ci` passed, including all 95 silent-host
fixtures, synthetic audio gates and strict Clippy
(`/tmp/riotbox-1570-ci-final.log`). Six new generated tests execute the exact
documented command with operator/Git adapters mocked, including rejection and
single-use behavior after early failure. Two independent command/contract
reviews and coordinator self-review retained zero P0–P3 findings. Native
[Rust CI 37972797712](https://github.com/marang/riotbox/actions/runs/37972797712)
passed on the exact pre-execution revision. These checks gated invocation;
their success did not predict successful real-host admission.

## Actual context and failure

The operator established a non-container, active local Linux user session:
UID 1000, `Active=yes`, `Remote=no`; server `PulseAudio (on PipeWire 1.6.9)`.
This was the real user session, not a sandbox-only audio failure.

`owned_modules()` failed before `creation_attempted` could become true:

`EvidenceError: malformed short module columns`

The frozen `_read_modules()` implementation splits short module text into
lines and requires each line to contain three tab-separated fields, allowing
one extra empty trailing field. This host's existing
`libpipewire-module-rt` arguments span multiple lines. In a separate read-only
post-execution capture, the module metadata had 41 text lines: six with three
columns, twelve with four, nineteen with one and four with two. Replaying that
captured text into the unchanged parser, without any host call or mutation,
reproduces the same error.

This is consistent with the upstream
[PulseAudio v17.0 short-module writer](https://raw.githubusercontent.com/pulseaudio/pulseaudio/v17.0/src/utils/pactl.c):
`get_module_info_callback()` emits the argument string directly, without
escaping embedded newlines. Actual `pactl` reports `17.0-98-gb096`, compiled
and linked with libpulse `17.0.0`. The established defect is an operator
metadata-framing assumption, not evidence of an AudioRuntimeShell failure.

- Owned null-sink load: not reached; no module ID or sink Node/serial admitted.
- CPAL 250-ms preflight: not reached.
- Three 60-second runtime runs: zero started, zero completed.
- Output geometry, callback progress, errors and callback-gap observation:
  unavailable; no driver transcript exists to validate.

## Cleanup and retained evidence

The operator published `result=failed`, `cleanup_verified=true`, with no
cleanup error. No own sink, audio child/group or stream had been created.
Separate read-only checks at approximately 18:31:54 UTC verified:

- Original default sink, mute and volume still equal the recorded baseline.
- The exact proposed owned sink Node is absent in validated PipeWire metadata.
- Its exact unique name is absent from short module metadata.
- No preflight or runtime was launched; the attempt directory initially
  contained only admission and result records.

No unload, default restoration, volume change or guessed resource cleanup was
performed. The post-execution check record has SHA-256
`ffd65475d1ab68562082effd496ddb71dffce26898c9db07ee10f382939e87e2`.

Bulky/local evidence remains ignored beneath the exact authorized phase root
`artifacts/audio_qa/local-silent-host-riotbox-1570-v2-2026-10-09`:
`prospective-binding.json`, `invocation-consumed.json`, and `attempt-01` with
admission/result, post-execution checks and captured module text. Result
SHA-256: `cc0a5e5f5eb93f83913193f4618d7efc8892becc89555a475ab24afb40aef80b`.
Captured module metadata JSON SHA-256:
`6707b140e6362bf7ad2a447f8c7bae51a18945e40a42add53a87772ee4f703a5`.
Execution log: `/tmp/riotbox-1570-host-once.log`. No V1 ignored evidence was
opened, hashed, changed or reclassified.

## Disposition and limits

This completes the authorized single attempt as negative admission evidence,
not a successful callback observation. There is no physical-device,
loaded-runtime, latency/xrun, endurance, source, musical, hardness or release
pass. No source/Holdout/commercial/capture audio, source-directory discovery,
physical output, DAW, TUI or human playback occurred. Existing unrelated
metadata was read only for admission and attributable post-execution checks.

[RIOTBOX-1571](https://linear.app/riotbox/issue/RIOTBOX-1571/handle-multiline-pulse-module-metadata-without-weakening-exact-owned)
owns a separate source-free, versioned metadata repair with generated
multiline/ambiguity and exact-identity regressions. It cannot authorize a
host retry. Any later real attempt needs fresh explicit authorization and a
new prospective owner/binding. RIOTBOX-1041 remains open.
