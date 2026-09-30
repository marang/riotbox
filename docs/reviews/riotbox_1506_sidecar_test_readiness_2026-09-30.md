# RIOTBOX-1506 — Analysis-timeout test readiness

Maintenance/regression, baseline `a7aad9e5`.
Native Windows run 36762259704, job 110047519117, failed only
`analysis_uses_separate_configurable_bounded_timeout`: the error operation was
Control, not the asserted Analysis. The one-second control policy was installed
before cold interpreter readiness. All new transport-deadline cases passed.

The existing synthetic hung-analysis fixture now deliberately delays startup
1.1 seconds, longer than that test policy, without reading source material.
The unchanged test then fails deterministically during Control on Linux too:
`/tmp/riotbox-1506-readiness-red-exact.log`, one executed test, exit 101.
An earlier unqualified exact-name invocation selected zero tests and is not
regression evidence.

The corrected test pings under the existing default bounded policy, verifies
protocol readiness, then installs its unchanged short control/analysis policy
and exercises the analysis timeout. This mirrors the existing positive
separate-budget regression. Production client behavior, default deadlines and
all source/DSP contracts remain unchanged. The technology-stack contract now
records this readiness convention so future short-budget tests do not become
cold-interpreter benchmarks.

All 24 Sidecar library tests and the unrelated-CWD integration test pass:
`/tmp/riotbox-1506-sidecar-green.log`. Strict Clippy passes
(`/tmp/riotbox-1506-clippy.log`) and full source-free `just ci` passes
(`/tmp/riotbox-1506-ci.log`): App 770, Audio 279, Core 470 and Sidecar 24,
binary/integration tests, Python/contract and synthetic audio QA, formatting,
tracked JSON and strict Clippy. Submitted-head Ubuntu/native Windows checks
remain required before integration.
No device, DAW, real source, holdout, commercial audio or human playback.

Sequential solo `code-review` / `code-review-rust` lenses cover the actual error
operation, fixture bounds, warmed handshake, unchanged policies and process
cleanup. No retained finding; no independent reviewer approval is claimed.
The fixed test cannot pass on a Control timeout and the fixture remains a
bounded local process reaped by the existing client lifetime.
