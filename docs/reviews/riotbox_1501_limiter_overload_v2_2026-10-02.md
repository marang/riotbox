# RIOTBOX-1501 — fixed-overload Development follow-up v2

Date: 2026-10-02. Base: `c4472c286469ce7f89f2d0b35d7df611007ae4ec`.
Classification: diagnostic maintenance/regression evidence, not instrument or
musical progress. Implementation and independent pre-access review are complete;
the accepted protocol is frozen under RBX-419 before any V2 original access.

## Authority and purpose

The user approved the separately frozen follow-up with the same three exact
Development originals. RIOTBOX-1501 moved from Todo to In Progress before the
new implementation branch. Linux only; Windows compatibility remains deferred.
No Holdout/commercial material, directory discovery, original-file access before
freeze, production-policy change or human playback is authorized.

V1's [retained result](riotbox_1501_limiter_development_comparison_2026-10-02.md)
has no source-backed limiter intervention even at 2x. The new
[protocol](../benchmarks/master_bus_limiter_calibration_protocol_v2.md) adds
one fixed 4x diagnostic input, retaining clean/2x as strict historical controls.
This is deliberately informed by consumed Development evidence; no blind,
source-general or perceptual validation is claimed. It addresses P023's narrow
question of protection behavior without silently increasing product gain.

## Pre-access design review

Independent Rust and Python design passes identified the same concrete output
budget risk (Performance And Operations lens): V1's 64-MiB limit was designed
for seven sample arrays, while V2 returns ten. At the maximum frame count,
ordinary f32 values serialized through JSON's f64 representation can exceed
that old limit. V2 preregisters 128 MiB; V1 is unchanged. A maximum-frame
generated serialization regression is required before execution.

The independent Spec And Evidence Auditor also required explicit historical
control identity, compiler scope, actual known widths, a named future condition
instead of the old positional index, and failure evidence across conditions.
The draft binds these before results: exact V1 metadata/f32 hashes, same native
compiler, PCM24/16/16, Sparse `stress_4x` frames [0,96000), and retained
preparation plus available clean/2x/4x diagnostics. These were design constraints,
not retroactive source-result adjustments.

## Execution state

At preregistration, no V2 original audio has been opened. Acceptance follows the
independent review below; final frozen checks and a clean implementation commit
must precede execution. No candidate audio or human claim exists. V1 artifacts
and contracts remain unchanged.

## Source-free implementation evidence

The shared Python owner uses closed V1/V2 entrypoints; Rust keeps the same
preparation and three-render comparison, adding the historical gate before any
4x buffer. The private protocol child owns the fixed historical identities,
and the test child has explicit imports. No Audio/App library, Cargo, production
limiter, capture preparation or V1 contract/result change is present.

- Rust: 18 generated calibration/preparation tests pass, including all three
  compiled control sets against the repository JSON and report-origin SHA;
  V1 byte-identical controls; direct pre×4 even when 2x was already limited;
  historical mismatch before 4x; retained failure/preparation evidence.
- Maximum 192000-frame, nine-output JSON: 74501149 bytes, above the old 64-MiB
  bound and below V2's 128-MiB bound. Complete failure reports with compacted
  oversized synthetic preparation remain below 64 KiB. The initial varying
  serialization fixture hit unchanged Source Monitor partition parity; the
  size-only test now uses constant generated PCM, without weakening that gate.
- Python: 23 generated/mocked tests pass, including compiler/metadata/contract
  rejection before access, the real access-owner loop stopping after a failed
  generated delivery, retained clean/2x/4x observations, and selection of 4x by
  condition identity rather than position.
- Feature/test Clippy with `-D warnings`, default-off build, mixed/extra CLI
  flag rejection and `git diff --check` pass.
- Metadata-only preflight passed for the exact Graph/Session and V1 comparison
  JSON pins; no source/capture audio was opened. Full source-free `just ci`
  passed (`/tmp/riotbox-1501-overload-v2-ci.log`); the subsequent test-only pin
  binding addition independently passed all 18 Rust tests and strict Clippy.

Main review identified a low-risk missing compiled-pin regression and a
wildcard test import (Test Strategist / Maintainability lenses). Both are fixed
in the test child and verified by the 18-test/Clippy run. They required no
algorithm, threshold, preparation, admission or measurement change.

## Final pre-access review

The independent Spec And Evidence Auditor / Adversarial Implementation Reviewer
reviewed the complete working diff and new files against `c4472c286469ce7f89f2d0b35d7df611007ae4ec`,
including Rust-specific correctness, access/failure boundaries, architecture,
tests and documentation. **Zero retained findings.** The reviewer independently
ran `just limiter-calibration-fixtures`: 9 Audio + 8 App + 18 runner + 23 Python
tests pass (`/tmp/riotbox-1501-v2-independent-review-fixtures.log`). A separate
reviewer of the Rust-only changes also returned zero findings and reproduced
the 16 comparison tests and 74501149-byte serialization proof (the other two
runner tests cover unchanged preparation).

The full review confirms historical hashes precede every 4x computation, direct
original-pre multiplication, V1 byte parity, metadata/compiler admission,
fixed condition-ID selection and fail-closed evidence retention. Main follow-up
self-review: **zero remaining findings**. Prior design/test findings retain
their stated origins and fixed dispositions; no source result informed a code
change. No new ActionCommand exists, so the five action surfaces are unchanged.

The accepted JSON SHA-256 is
`e30a439dbc0d61245cb56ad099c4110481342a0bad0e4a3426125cfcaaeae9af`.
After pinning, metadata-only preflight and the complete 9/8/18/23 synthetic
fixture suite passed again (`/tmp/riotbox-1501-overload-v2-frozen-fixtures.log`).
No result-driven contract changes are permitted after the next source access.
