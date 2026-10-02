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

## One bounded execution

Execution used clean implementation commit
`0bb7a17b72f15addb3da38748a13a36be9ea5371`, after acceptance and all pre-access
checks. The runner rebuilt the locked native Linux feature target and bound
executable SHA-256
`573faddf9e9e418b6f7a0509de10e847112ac410ae7e30c09fd0574244d4dde0`.
The exact compiler equals V1's pinned identity. No implementation, contract,
parameter, window or admission changes followed source results.

Access session `1ecaadb0-30f8-44c3-8be2-5fc33f594806` ran from
`2026-10-02T10:02:40.945290Z` to `2026-10-02T10:03:03.750437Z` and completed
successfully: three exact original admissions and owner deliveries, no directory
discovery. The access layer correctly says qualification is not evaluated by
that layer; the comparison report owns the actual technical result.

Local ignored metadata, retained without audio hydration:

- `artifacts/development/riotbox-1501/calibration-v2/comparison.json`:
  `cfbe5eae3848503f8a906b4a4ea62cf6ad9cfebd25c7f6a4e583fdf3cb6f8ed2`.
- `artifacts/development/riotbox-1501/calibration-v2/development-access.json`:
  `f633731e056a9bc3daa9a23b142b88f8854c8d32d92177d5f28da13b4d44f32d`.

The successful nine mix passes yield 27 policy outputs and three ordinary
artifact-backed captures, with six/six/eleven committed preparation actions and
their matching commit records. All repeat-128, partition-257 and baseline-API
controls pass. Every clean/2x input and output matches the frozen historical
f32LE identity, has zero limiter writes and remains unclipped. V1 comparison
and access JSON hashes remain exactly unchanged.

## Fixed 4x observations

Counts are individual interleaved samples, not frames or events. Input clips
mean `abs(sample) >= 1`; this deliberately overloaded diagnostic input is not
an assertion that the unchanged ordinary product clips.

| Case | 4x input peak | Input clip count | A writes | B writes | C writes | All output clips |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Dense | 1.5995879173 | 1520 | 4984 | 3208 | 5024 | 0 |
| Tonal | 1.0745946169 | 8 | 104 | 32 | 110 | 0 |
| Sparse | 1.5667495728 | 460 | 964 | 728 | 972 | 0 |

All outputs stay within their declared f32 sample-peak ceilings. Dense/Sparse
reach approximately 0.9850000143 for A/B and 0.9524999857 for C; Tonal peaks are
0.9838923812 / 0.9849645495 / 0.9524951577. These are sample-peak observations,
not true-peak, device or hearing-safety guarantees.

B and C differ from A in every whole-render 4x case. This does not establish
audibility, distortion preference or musical quality. Whole-render f64 delta
RMS relative to A is Dense B/C 0.0005231429/0.0022410337, Tonal
0.0000557992/0.0001812127, Sparse 0.0001785964/0.0011311950. The persisted report
separately retains original Rust f32 metrics; these descriptive f64 values do
not replace them or redefine the product's numerical contract.

Fixed local windows are reported without adaptation: Tonal's attack window has
zero writes under every policy, despite its whole-render intervention. Dense
and Sparse have writes in all three local windows; Tonal has writes in body and
recovery. No onset detector, peak search or window substitution was introduced.

The preselected possible future comparison is specifically Sparse `stress_4x`,
frames [0,96000), two seconds at 48 kHz. A/B/C modify 766/544/772 samples there;
B and C are each non-bit-identical to A in that same window. Thus it contains
actual policy differentiation, unlike V1's identical controls. No comparison
WAV was generated or played; `artifact_generated=false`,
`human_verdict=unverified`, `quality_proof=false` remain correct.

## Interpretation and remaining boundary

Status: `technical_comparison_complete_no_policy_selection`. This phase proves
bounded source-backed sample-peak protection under the declared 4x diagnostic
challenge, while preserving historical controls. It does not calibrate a
perceptually preferred policy or qualify source intelligence, hardness or a
release. Production stays provisional A with unchanged gain and parameters.

The V2 one-session source budget is consumed. Do not rerun or extend it. A later
comparison artifact would need its own explicit bounded artifact/source phase,
unchanged preselected interval, exact technical preflight and fresh human
readiness under the listening-review workflow. No source/capture opens, implicit
playback, adaptive gain or additional candidates are authorized by this result.
RIOTBOX-1501 remains open; this is a completed technical slice, not full issue
completion. PR/remote CI and merge closeout follow.

## Independent result audit

The independent reviewer read only the two exact V2 JSON artifacts plus tracked
contracts/code, without audio/capture/binary hydration or re-execution. Raw
report/access hashes, shared session ID, compiler/build/protocol identities,
three admissions, preparation/commit ownership, all 27 outputs and historical
control fingerprints match. The reviewer independently confirmed the 4x table,
unobserved Tonal attack intervention and actual differentiation in the fixed
Sparse future window. **Zero findings.** The access layer's non-qualification
status is correctly separate from completed technical comparison; no metric or
artifact-availability flag is promoted to a human verdict. Main final self-review
also retains zero findings; post-access changes are result documentation only.
