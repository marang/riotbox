# RIOTBOX-1577 — generated loaded-Session preparation review

Date: 2026-10-15. Base: `8e0aeab5a269e78b42f46bcdea13a7e07d1775b0`.
Scope: the complete branch diff against HEAD, including new untracked owners,
tests and contracts before the first commit. Classification: maintenance/regression
for the P023 recording reliability gap, not new audible instrument progress.

## Delivered contract and ownership

[V1](../benchmarks/generated_live_master_load_fixture_v1.md) / RBX-443 admits
only one generated profile into an exclusive fresh owner. Actual App/Sidecar
ingest, the shared existing Core-committed prefix, save and ordinary restore
remain authoritative. Six existing recording-plan/offline-mixer windows cover
2/8/16 bars at generated 44.1/48-kHz stereo geometry. Complete computed metrics
precede aggregate failure; structural identity errors still stop immediately.
No new ActionCommand, persistent App state, analyzer, recorder, runtime, DSP,
production policy/threshold, recording version or dependency was introduced.

The extraction preserves legacy timing preflight, action IDs/timestamps, family
checks, logging, render-plan helpers and later journey. New semantic owners are
under 500 lines; legacy `live_flow` shrinks from 1,313 to 1,086 lines. Its
existing cohesive journey is not mechanically resharded for line count.

## Pre-PR findings and dispositions

- **P2, fixed — offline publication ownership.** In the WIP `windows` path,
  the general-purpose WAV writer could overwrite an occupied destination,
  contrary to the preparation's explicit fail-closed occupied-output contract.
  Spec And Evidence Auditor and the coordinator's Adversarial Implementation
  lens independently identified the gap. `windows::publish_offline_wave` now
  writes only its own exclusive temporary file, verifies decoded geometry and
  hashes the same readback bytes, then uses no-clobber publication. Generated
  occupied/link/missing-parent controls prove preservation and retained metrics;
  the real matrix's six collision failures retain all computed reports and
  leave earlier artifacts unchanged. This is a WIP contract repair, not a
  demonstrated data-loss regression on shipped main.
- **P2, fixed — actual producer attribution.** Adversarial Implementation
  Reviewer / Rust API review traced post-execution script hashing and reopening
  the executable pathname: an ordinary concurrent replacement build/edit could
  attribute different producer bytes. `generated_load::prepare` now executes an
  exclusively retained snapshot of the exact generator bytes hashed and hashes
  the running Linux executable inode through `/proc/self/exe`. The full roundtrip
  asserts both bindings. The original evidence is static, not an executed
  RED/GREEN replacement reproduction; hostile concurrent owner mutation remains
  outside the documented contract.

Both candidates are resolved before PR. No retained P0–P3 findings remain and
no finding was deferred. Computed-window aggregation was also made explicit;
no technical threshold or production algorithm was tuned after results.

## Verification and attribution

- Default diagnostic binary: **30 PASS**, 19 existing + 11 new. The full
  generated ingest/commit/save/restore, six successful windows, source/capture
  PCM mutation and missing-file rejection, and six retained collision failures
  are assertions inside one new test, not additional independent test counts.
  `/tmp/riotbox-1577-final-focused.log` (74.32 s).
- Opt-in calibration selection: **18 PASS**, 30 filtered, preserving the existing
  feature path. `/tmp/riotbox-1577-calibration-regression.log`.
- Generator selector/default-corpus/reproducibility controls: **2 PASS**.
  `/tmp/riotbox-1577-generator-tests.log`. Default retains all twelve generated
  cases and the selected source is byte-identical to its default-corpus member.
- Scoped all-feature strict Clippy: **PASS**.
  `/tmp/riotbox-1577-final-clippy.log`. JSON parse and diff check: **PASS**.
- Actual new preparation CLI: **PASS**, six of six windows, ordinary restore
  verified, one identity-bound capture, no host/controller execution or quality
  claim. `/tmp/riotbox-1577-cli.log`, fresh owner
  `/tmp/riotbox-1577-cli-Y6Qrtl/prepared/preparation.json`.
- First source-free local `just ci`: all tests and synthetic audio PR controls
  passed, then `decision-search-fixtures` could not create a Just temporary file
  under sandbox-read-only `/run/user/1000/just` (OS error 30). This is an
  execution-context limitation, not a code failure. The unchanged complete gate
  **PASSed** with normal user filesystem access, including tracked JSON/diff
  validation and strict all-target/all-feature Clippy. Logs:
  `/tmp/riotbox-1577-full-ci.log` and `/tmp/riotbox-1577-full-ci-user.log`.
  Native PR checks still gate merge.

The first full new test retained an extra large App facade in its own frame
while calling production preparation/measurement and overflowed the default
test-thread stack. Moving only that test's restore scope into a scalar/identity
helper made the same full test pass without a larger stack, blanket heap bypass
or production-state change. Initial four compile-time assumptions and this test
correction are not product or host regression evidence. The historical WIP
pause was overcautious; it is not an ongoing blocker or model-quality finding.

Independent **Spec And Evidence Auditor** reviewed the new preparation, tests,
contracts, provenance, admission and claim boundaries: zero retained findings;
independently ran the two Python controls and diff check. That reviewer's prior
`ready_prefix`/`live_flow` authorship is excluded from independent review credit.
Independent **Rust/API, Adversarial Implementation Reviewer, Maintainer and
Product Pragmatist** reviewed the complete extraction and new owners: zero
retained findings; independently ran ten small Rust controls and two Python
controls. Logs: `/tmp/riotbox-1577-independent-spec-generator.log`,
`/tmp/riotbox-1577-independent-small-tests.log`,
`/tmp/riotbox-1577-independent-python-compat.log`. Those reruns overlap the root
tests and are not additive coverage. Coordinator verified dispositions, exact
source hashes, caller/consumer contracts and performed a short self-review:
zero additional findings. Native PR checks and closeout remain coordinator gates.

## Limits

Fixture origin, false quality/source-intelligence/host-execution claims and a
named five-contributor composite are mandatory. No real source/capture/Holdout/
commercial audio, source-directory discovery, old ignored attempt, actual audio
runtime/device/service/host metadata, DAW, playback, broad QA or Windows expansion
was used. No human verdict is requested or inferred. The normal source-free CI
synthetic packs are technical controls only.

Future loaded-host execution needs a separate prospective versioned contract,
exact reviewed revision/native CI/binary binding and fresh explicit authorization.
Consumed host phases stay consumed; RIOTBOX-1572's ALSA trigger remains unexplained.
RIOTBOX-1041 and RIOTBOX-1036 remain open. No musical, source-general, physical
device, latency, endurance, hardness or release qualification follows.
