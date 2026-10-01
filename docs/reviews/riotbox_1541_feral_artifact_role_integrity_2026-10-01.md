# RIOTBOX-1541 Feral artifact role integrity

Date: 2026-10-01
Implementation baseline: `57fa0184c06257a93434454614b72a995a4c9a5b`.
Integrated archive-only predecessor: `1f2f7f647d9a09e67193c6da24d111c009b2978a`.
Decision: RBX-410
Classification: maintenance/regression, independently confirmed QA corruption.

The existing Feral pack now rejects physical coupling among its nineteen
artifact destinations before analysis/rendering or any write. This removes a
P023 evidence-trust blocker: named stem roles must not silently contain another
role's output. Valid rendering remains byte-identical; this is not musical
progress, source qualification, a human verdict or release readiness.

## Reproduction and correction

Two actual CLI probes on integrated main accept hardlinked TR-909/W-30 stem
destinations and return zero, leaving identical W-30 bytes under both names.
Separate files yield distinct stem hashes; the synthetic input stays intact.
That distinguishes output-role coupling from source loss and renderer failure.

The first public CLI regression fails on replaced TR-909 output, then passes
after the plan-local guard. The original unattended loop against the exact
fresh candidate now rejects with input and both previous outputs unchanged:
`bash /tmp/riotbox-feral-output-alias-probe.2dMzv8/repro.sh alias /home/markus/Dev/riotbox/target/debug/feral_grid_pack`.
Its separate mode still returns zero with distinct stems. Every probe invocation
owns a fresh temporary directory and uses one exact generated synthetic input.

## Ownership and coverage

The existing typed PackOutputPaths, now 90 lines, exposes one private artifact
iterator for both source preservation and mutual-output checking. The new check
admits all entries with RBX-409's shared absent/regular helper, then delegates
each existing-file pair to the unchanged read-only physical identity guard.
The bounded layout permits at most nineteen paths and 171 pair comparisons.
The sole renderer integration line follows the existing source-format/source
guard and precedes analysis. Writers and manifest paths retain the same plan.
No second filename catalog, filesystem backend, framework, dependency, target,
public library interface or product-state owner is added.

Five new CLI tests in a 194-line owner cover 180 actual invocations: all 171
hardlink pairs; six representative audio/metrics/metadata Unix symlink cases
in both directions; a relative stem link through an aliased output directory;
late manifest/README coupling without early publication; and independent
equal-content output success. The successful control decodes all eight WAVs
and checks distinct TR-909/W-30 bytes. Literal expected paths are independent
of the production planner; no private preflight mocks. All 32 previous source,
comparison and W-30-output safety test bodies remain untouched.

## Verification and review

All 138 focused names/statuses match and pass in Debug and Release: 95 binary
units, six capacity/duration/verification cases, 32 previous safety cases and
five new tests. Standalone fresh five-binary build precedes sequential CLI
captures; each capture completes before its hash comparison. All 141 records
and 85 input/output hashes match the retained baselines across Feral (31/39),
W-30 (32/9), Before/After (31/29) and comparison (47/8). Both existing comparison
manifests validate. No algorithm, threshold or contract is tuned to results.

Sequential code-review/Rust/design/spec/evidence lenses inspect complete
changed functions, callers, actual writers, layout identity, missing/error and
late-publication behavior, test independence, compatibility and module growth.
No actionable finding remains in this diff after short self-review. This is a
solo review, not an independent reviewer panel.

First red/green and final cases: `/tmp/riotbox-1541-output-safety-{red,green,final}.log`.
Focused tests: `/tmp/riotbox-1541-focused-{debug,release}.log`.
Fresh build: `/tmp/riotbox-1541-build-reviewed.log`.
Full source-free CI: `/tmp/riotbox-1541-ci-final.log`, actual exit zero.
Warning/error scan, fmt/diff/include gates and targeted RBX-410 readback pass.
Exact-head native PR CI/review, merge/main sync and archive/cleanup remain
separate obligations, not inferred from local checks.

Stable filesystem namespace only: directories may be created before rejection;
existing identity/readability/unsupported-backend errors fail closed. No hostile
namespace replacement protection, atomic whole-pack or power-loss guarantee,
Windows Audio/filesystem/device execution proof or inferred mutual-output fix
for other renderers. No Core/Session/replay/runtime/DSP/algorithm/threshold/schema
or frozen Stage-A change. No real Development/Holdout/commercial audio, source
directory discovery, DAW/device/playback, subagent, generated product fallback
or source/musical/hardness/human/release claim.
