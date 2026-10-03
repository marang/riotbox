# Bounded CPAL probe admission — RIOTBOX-1565

Date: 2026-10-03. Comparison: `2fe224692a5217303c190b63b58aa4ca81e05ec6`
(main after PR #1667) to this branch. P017 maintenance of the existing P023
callback/host-preflight path, prerequisite for RIOTBOX-1041; not a host experiment
or an audible vertical slice.

## Diagnosis and contract

The historical `cpal_spike` entry point prints its summary and returns unit even
when runtime startup returns a handled error. That control flow yields exit 0;
this is static evidence, not an executed device-failure reproduction. Separately,
the old health projection accepts Running/Stopped with zero callbacks. A pure
generated-health RED reproduced that false positive without opening a device.

RBX-435 and Audio Core §15 freeze the bounded observation contract before the
production repair. Success requires Running/Stopped, at least one callback,
zero stream errors/scratch overflows and no retained stream-error detail. Failure
precedence is stream error/detail, scratch overflow, lifecycle, then absent
callback observation. The summary preserves both existing error counters and
diagnostic context. No new telemetry producer or persisted state is introduced.

The CLI still prints the summary, but returns success only for typed `Ok`;
`Failed` and `NotRun` return failure. The 250 ms duration, silent default runtime,
callback implementation and snapshot-before-stop lifetime are unchanged. A
zero-callback window fails the observation gate, not the backend itself. Maximum
callback gap remains informational with no new threshold.

## Verification

- Baseline RED: the generated zero-callback health test unexpectedly obtained
  `Ok`. Initial GREEN: the same regression passes after the admission repair.
- Six pure-health tests cover Running/Stopped with zero or observed callbacks,
  Idle/Faulted, error counter/detail independently and together, scratch
  overflow, failure precedence and all five startup-error variants. Nested
  cases preserve error evidence across lifecycle and callback-count changes.
  A clean snapshot with `u64::MAX` gap still passes: no gap threshold is implied.
- One binary-unit reporting test maps generated `Ok`, `Failed` and `NotRun`
  summaries to exit codes. It invokes only `report_summary`, not `main` or CPAL.
  The baseline had no reporting seam; this is a new exit-propagation regression,
  not a claimed baseline subprocess RED or real startup-failure experiment.
- Complete `cargo test -p riotbox-audio` passes: 333 library tests, the new CLI
  unit test, remaining binary and integration suites. One pre-existing manual
  release-mode snapshot benchmark remains ignored. Generated fixtures only.
- Formatting, diff check and full source-free local `just ci` pass, including
  workspace tests, Python/contracts, generated-audio smokes and strict Clippy.
  Native exact-head CI remains a separate merge gate.

Coordinator logs: `/tmp/riotbox-1565-red.log`, `/tmp/riotbox-1565-green.log`,
`/tmp/riotbox-1565-audio-tests.log`, `/tmp/riotbox-1565-ci.log`.

## Review and ownership

Independent Adversarial Rust Reviewer retained no P0–P3 findings. Inspected the
complete production delta, consumers and runtime health/telemetry producers,
contracts and baseline RED evidence. Independently ran six exact pure-health
tests and the one CLI reporting test; did not rerun baseline RED or full CI.
Logs: `/tmp/riotbox-1565-review-probe-exact-tests.log` and
`/tmp/riotbox-1565-review-report-exact-tests.log`. An initial substring filter
also selected five generated-WAV timing tests; all passed, and the corrected
run excluded them. Neither run used real sources, a device or playback.

Independent Spec and Evidence / Product Pragmatist Reviewer retained no P0–P3
findings, including this report's evidence boundaries. Independently executed
the six exact health tests and reporting test through the existing compiled test
harnesses, not fresh Cargo builds. Inspected coordinator RED/GREEN/audio logs and
ran diff check. Full CI remains coordinator evidence; neither reviewer ran a
device experiment. Reporting log: `/tmp/riotbox-1565-spec-evidence-exit.log`.

Coordinator self-review retains no correctness, contract, ownership or missing
test finding. Full local CI is green; exact-head native CI still gates merge.

`probe.rs` remains one cohesive 344-line owner including tests; the CLI is
54 lines. No split, new dependency, ActionCommand, JamAppState field, alternate
health model or string-controlled branching is needed. Outcome branching uses
the existing typed enum; reason strings are diagnostic only. Queue, commit,
Session and replay surfaces are inapplicable because no product action changes.

## Limits

This establishes health-projection and process-status semantics only. It does
not establish continuous callback activity, xrun absence, latency, route
isolation, physical audibility or endurance. Historical real-session/sandbox
observations remain unchanged. RIOTBOX-1041 still requires its own real-session
experiment; success here does not complete it or any release gate.

No actual `cpal_spike` execution, device/runtime/DAW start, human playback, real
source/Holdout/commercial/capture audio access, source-directory discovery,
DSP change, new timing knob, TUI or Windows expansion occurred. No listening
review is needed because this maintenance slice changes no produced audio and
runs no human playback; it grants no musical, source-general or hardness claim.
