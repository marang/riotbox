# RIOTBOX-1524 — Architecture checkpoint after QA ownership changes

Date: 2026-10-01. Integrated baseline:
`91299e2c7a4b1bf363b563fdd2398154796e13b6` (RIOTBOX-1523, PR #1590).
Classification: maintenance/regression. This is a solo, risk-directed
`review-codebase` checkpoint, not an exhaustive audit or independent panel.

The five substantive ownership slices since RIOTBOX-1516 are lane-recipe QA
(1517), W-30 preview QA (1519), Feral before/after orchestration (1520), numerical
Feral evidence (1521), and bounded Feral timing ownership (1523). Behavior fixes
1515, 1518 and 1522, test-only changes and archive branches do not count. RBX-397
explicitly requires this checkpoint before another structural migration.

## Findings and priorities

**P2 — Unrepresentable Feral grid capacity is accepted, RIOTBOX-1525.**
Review lens: Adversarial Implementation Reviewer. The actual CLI parser accepts
finite positive `--bpm 1e-38 --bars 2`. In
`feral_grid_pack/grid.rs:23,52–53`, cumulative frame calculation rounds an `f64`
then casts to `usize`, saturating at `usize::MAX` on this 64-bit host. `Grid::new`
returns success even though stereo sample count cannot be represented. Actual
consumers allocate using unchecked multiplication in `render_stems.rs:44` and
`mc202_bass_pressure.rs:202–203`.

A pure probe imports the unchanged actual Args/Grid/config leaves, parses those
arguments and evaluates only the consumer's scalar multiplication. Debug with
overflow checks produces a caught arithmetic panic; optimized overflow-disabled
execution wraps to `18446744073709551614`. No source is opened, no renderer is
run, and no huge allocation is attempted. The complete downstream allocation
failure is inferred from the code, not an executed memory-stress result. The
normal 128-BPM two-bar grid remains 165375 frames/330750 stereo samples.
Existing checked beat-count multiplication correctly rejects `4 * u32::MAX`;
that initially considered candidate is rejected, not a second finding.

RIOTBOX-1525 is Medium/P2 Todo. Reject unrepresentable frame, interleaved sample
and `f32` byte capacities through the existing Result before rendering or
allocation. Preserve cumulative rounding and ordinary output; add no arbitrary
musical BPM threshold or general memory-budget policy. Regression tests should
exercise the pure seam in Debug and actual Release before any invalid CLI
render attempt. This is a pre-existing local robustness defect, not a regression
introduced by the five ownership migrations, RCE evidence, or a musical failure.

No additional demonstrated P0–P3 correctness finding survives the sampled
checkpoint. No rewrite or second product-state owner is justified. Fix the
demonstrated capacity error before optional further QA ownership migration.
RIOTBOX-1509 remains open: green Windows transport runs do not establish a
causal fix for its intermittent startup failure.

## System model and ownership

| Owner | Responsibility and dependency boundary |
| --- | --- |
| Core | Source Graph, Session/action/replay and source timing remain product truth. QA consumers cannot promote their reports into readiness authority. |
| App and Audio | Verified source/capture admission and prepared control-side projections feed the existing mixer. No QA owner enters the realtime callback. |
| Lane-recipe QA | Argument/case data, preparation, render, metrics and pack presentation remain one existing binary with explicit owner imports. Primitive controls grant no source-intelligence or human pass. |
| W-30 preview QA | Arguments, exact input/metrics, rendering and comparison retain separate owners. Preview comparison consumes output rather than becoming source policy. |
| Feral before/after QA | Preparation and comparison orchestration consume existing render/evidence owners; publication remains a bounded QA operation, not a second Session transaction. |
| Feral grid QA | Args/config feed conservative BPM/profile policy; pure evidence adapts Core probes; analysis feeds groove and readiness presentation. Root compatibility imports preserve unconverted consumers without reverse dependencies. |

## Boundary review

The numerical evidence children remain leaves over existing typed probe data.
Timing/profile owners form an acyclic dependency graph: Args depends on config;
BPM on Args/config; profile and pure evidence are leaves over Core; analysis
consumes evidence/profile/config; groove consumes BPM/evidence/config;
presentation consumes BPM/evidence/profile. None imports the root facade,
orchestration or manifest owner. Shared visibility is binary-private; scalar,
anchor and serialization internals stay private. The 21 remaining textual
includes/two owners are explicitly inventoried, not silently exempted.

Current persistence, source/capture identity, Sidecar and actual callback paths
were sampled again. An exact Git comparison against RIOTBOX-1516's merged
checkpoint `7ca8628e8ba078a103c7b1f295f20cd03ffc1c16` is empty for Core, Sidecar,
Audio runtime/source cache and App persistence/capture artifacts. Session
publication stays the commit point over exact hash-bound generations. Recovery
save uses the on-disk Session's authoritative generation; unrelated alias I/O
errors remain errors. Original audio decode/hash share one byte buffer. Capture
identity and explicit legacy adoption stay Core-owned, with no invented old hash.

The actual callback retains preallocated scratch, oversized-buffer silence and
telemetry, prepared snapshot mixing, source-monitor policy, limiter, capture
and output ordering. No blocking file/hash/analysis/model work was introduced.
Sidecar retains its absolute pipe-I/O deadline and invalid-peer cleanup. These
reads and unit fixtures do not prove real-device behavior or filesystem support.

## Fresh integrated evidence

All targeted tests below passed on the integrated baseline:

- Five affected QA binaries: **84 unit tests plus three CLI integration tests**
  (87 total), `/tmp/riotbox-1524-qa-tests.log`.
- Core library: **470**, `/tmp/riotbox-1524-core.log`.
- Graph transaction/recovery: **16**, `/tmp/riotbox-1524-persistence.log`.
- Capture identity: **8**, `/tmp/riotbox-1524-capture.log`.
- Original-source admission: **3**,
  `/tmp/riotbox-1524-source-admission-final.log`. An earlier wrong filter ran
  zero tests and is excluded from proof; these are actual executed cases.
- Callback scratch: **2**, `/tmp/riotbox-1524-callback.log`.
- Sidecar: **24 library tests plus one working-directory-independent process
  integration**, `/tmp/riotbox-1524-sidecar.log`.
- Verification command contract: **two Python tests**, including the local
  Just argv probes, `/tmp/riotbox-1524-verification-argv.log`. GitHub's Windows
  job remains transport-only; native CI without Just skips the local-only probe.

Fresh execution of all **31 CLI stdout/stderr/status cases** is identical to
RIOTBOX-1523. At the exact named synthetic control paths under
`/tmp/riotbox-1523-byteproof.uYJSWo`, all **39 hashes** remain identical, including
16 output WAVs and the generated four-second input. No source directory was
searched. These are technical-only QA reruns, not new artifacts for listening,
and grant or transfer no human verdict.

The capacity probe source and Debug/optimized outputs are retained under
`/tmp/riotbox-1524-grid-probe.tKTtr7` and
`/tmp/riotbox-1524-grid-{debug,release}-final.log`. Its expected caught Debug
panic is defect evidence, not a green validation log. Full source-free `just ci`
passes with verified exit status zero in `/tmp/riotbox-1524-ci.log`. All final
validation logs listed above and that CI log were explicitly scanned
warning/error-free; the expected diagnostic panic is excluded. This docs-only
branch's native CI/merge remain separate completion gates.

## Coverage limits and disposition

No real Development source, Holdout, commercial reference, audio device, DAW or
playback was accessed. This sampled audit excludes whole-repo line-by-line
review, decoder/numeric fuzzing, huge-memory allocations, filesystem/OS matrices,
power-loss and concurrent-writer durability, allocation tracing and actual live
terminal/device use. Regular-file reads and Sidecar frames remain uncapped;
pipe deadlines do not bound JSON CPU, spawn, kernel stalls or descendants.
Finite representable but impractically large renders remain a separate resource
policy question; the confirmed defect is mechanical representability.

AGENTS routes periodic reviews to the workflow core, which does not currently
state a numeric cadence; this checkpoint follows RBX-397's explicit five-slice obligation,
not an invented global rule. No new architecture/algorithm/threshold decision
arose, so routine checkpoint evidence adds no Decision Log entry. Keep the
capacity fix in RIOTBOX-1525 and the unresolved startup diagnosis in 1509.
Reset this checkpoint's structural slice count only after merge and closeout.
This maintenance grants no P023 musical, hardness, source or release claim.

Sequential branch-level `code-review` of this report against the integrated
baseline checked factual scope, exact finding locations, executed test counts,
source boundaries and disposition. The caught scalar panic is not mislabeled
as a full renderer run; the wrong zero-test filter is excluded. Draft line
references and the cadence routing statement were corrected before delivery.
No remaining finding applies to this docs-only diff; follow-up self-review
also has zero findings. The separately tracked production defect is deferred
only to the immediately following RIOTBOX-1525 bug slice, not declared fixed.
