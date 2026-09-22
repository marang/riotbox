# RIOTBOX-1498 — Separate fixture readiness from operation timeout testing

Classification: maintenance/regression, test-only. Base `a361864f`. This removes
an independently reproduced local-CI interruption found during RIOTBOX-1491 and
RIOTBOX-1412. It changes no production protocol, timeout policy or audio path.

## Diagnosis and red/green evidence

The unchanged `analysis_can_outlive_control_budget_without_timing_out` test
installed its deliberately short 50 ms Control timeout before the first
protocol handshake. `build_source_graph_stub()` calls that handshake when the
client is not yet ready. Therefore interpreter startup/scheduling could fail
the test before its intended 75 ms Analysis response (250 ms budget) began.

A single-test stress loop (96 invocations, at most 32 concurrent processes)
failed with Control timeouts in 96/96 and 93/96 runs. Eight-worker runs passed.
The hypotheses were startup charged to Control, a misclassified Analysis
response, or exhaustion of the Analysis budget. The observed operation and the
warmup experiment distinguish the first from the other two.

The fixture now deliberately waits 100 ms before answering the initial Ping.
With this regression fixture but before the Rust correction, one ordinary test
invocation reliably fails after 50 ms with `sidecar control operation did not
reply within 0.1s` (the existing message rounds to one decimal place).

The test now calls `ping()` under the existing bounded default Control timeout,
then installs its unchanged 50/250 ms operation policy. The analysis request
still has to outlive the shorter budget and return the expected typed fixture
error within its own budget. The same stress loop subsequently passes 96/96
twice, including the intentional slow startup. The 14-test Sidecar suite passes;
the separate 50 ms hung-Control and hung-Analysis tests remain unchanged.

Resolve the current executable from Cargo metadata rather than retaining a
stale hash-named test binary when switching between workspace and package builds:

```sh
sidecar_test_binary=$(cargo test -p riotbox-sidecar --lib --no-run --message-format=json \
  | jq -r 'select(.reason == "compiler-artifact" and .target.name == "riotbox_sidecar" and .profile.test) | .executable')
test -x "$sidecar_test_binary" || exit 1
seq 1 96 | xargs -P 32 -I '{}' "$sidecar_test_binary" \
  --exact client::tests::analysis_can_outlive_control_budget_without_timing_out --nocapture
```

An initial post-fix probe accidentally reused the pre-fix workspace executable;
it was rejected as stale-artifact evidence. Both reported green stress runs use
the metadata-resolved current executable. No instrumentation or debug helper is
left in production code. This bounded test fixture does not establish a general
scheduling or performance guarantee under arbitrary system load.

## Review and verification

Solo code-review/Rust review covers the test setup, fixture protocol sequence,
unchanged production policy and retained negative-timeout cases. Zero retained
findings; no new dependency, action, persistence contract or runtime state. A
module split is not necessary for this narrowly scoped existing-test correction.
Full normal-parallel source-free `just ci` passes, including Rust/Python,
synthetic audio/observer gates, format and strict all-target/all-feature Clippy.

No real source file, Holdout, commercial reference, DAW or human playback was
accessed. The SourceDescriptor path in the test is metadata only; the fixture
does not open it.
