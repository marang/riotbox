# RIOTBOX-1538 W-30 comparison input preservation

Date: 2026-10-01
Baseline: `e393456372d75ef295395bfb6169d46c96d4bdb6`
Integrated Before/After predecessor: `468a9e9e9ca4d08bca76bedfc5372120424a5588`
Decision: RBX-408
Classification: maintenance/regression, existing offline QA input destruction

The W-30 metrics comparator could return success after replacing the WAV or
metrics it describes with its own report. This slice protects those inputs and
the two output files before publication. It is not audible product delivery or
source/hardness qualification.

## Cause and actual CLI regression

Two pre-fix CLI runs with report equal to baseline.wav replace 17684 bytes of
exact generated WAV with 993 bytes of Markdown while returning zero. A metrics
collision likewise changes 680 to 993 bytes with success. A separate report
preserves the WAV. The comparator parses metrics, writes its report and only
then checks manifest artifact existence; existing paths can therefore pass
while containing the wrong bytes.

The public CLI regression first fails on replaced input bytes, then passes.
Two further red/green cycles cover report/manifest self-replacement and a
nonregular associated WAV rejected only after early report publication.
The original loop now returns CLI exit 1 with unchanged 17684-byte WAV and
680-byte metrics for the corresponding collisions:
`bash /tmp/riotbox-compare-input-collision.wVxZFG/repro.sh collision` and
`bash /tmp/riotbox-compare-input-collision.wVxZFG/repro.sh metrics`.
Its optional second argument pins the comparator executable.
Only explicitly generated no-source QA controls are used, not product fallback.

Linear In Progress precedes the independent main-based branch and red test.
The existing shared helper was already merged; the disjoint Before/After merge
is fast-forwarded without discarding this slice's changes. No feature stacking,
backfilled issue, unrelated edit or subagent.

## One bounded preflight and existing naming ownership

The ordinary binary-private output_safety owner is 45 lines. Both metrics are
parsed first; help and existing argument/metrics errors keep their precedence.
Before printing or writing the report, all four referenced inputs must be
regular and readable/identifiable. Both actual outputs are checked against
both metrics and their inferred WAVs using the unchanged qa_source_safety
physical identity implementation. Writers and preflight reuse the existing
manifest_path_for_report_path and audio_path_for_metrics_path naming helpers;
there is no second safety-only filename convention or generic pack framework.

Identical report/manifest names reject even when absent. When report exists,
the same guard checks the manifest against its physical identity too.
Only genuinely absent output entries can skip identity inspection; unknown,
dangling or nonregular destinations reject. Input inference follows the metrics
pathname convention, including when the metrics itself is a symlink.
No existing renderer, comparison, parsing, formatting, manifest or config body
is modified. No public library, target, dependency, product model or realtime
use is introduced.

## Technical proof and compatibility

Eight new actual CLI tests cover:

- All four direct report/input collisions, preserving both metrics and WAVs.
- Both outputs hardlinked to any input, preserving every previous output.
- Unix output/input/directory symlinks.
- Late manifest/input collision with an absent report, without early publication.
- Nonregular and Unix dangling report/manifest destinations, no target creation.
- Missing or nonregular associated WAVs, no partial output publication.
- Distinct equal-content outputs and shared baseline/candidate metrics succeed.
- Absent identical output names and existing report/manifest hardlinks reject.

All 117 focused test names/statuses match and pass in Debug and Release:
85 existing binary units, six capacity/duration/verification integration cases,
18 unchanged renderer safety cases and eight new comparator cases.
No test is removed or ignored. The existing shared helper and all three renderer
safety test files remain unchanged; assertions cross the actual public CLI seam
without inspecting or mocking private preflight internals.

All 47 ordinary CLI status/stdout/stderr records match the built baseline,
including help, invalid/missing arguments, missing metrics, normal pass and
explicit drift failure. All eight input/report/manifest hashes match. Both
normal and drift-fail manifests validate existing referenced artifacts.
The exact synthetic-only parity directory is
`/tmp/riotbox-1538-byteproof.JIvtUE`. No thresholds are changed against results.

## Solo review and limits

Sequential code-review and Rust lenses inspect complete functions, both write
consumers and naming helpers, input/metrics error ordering, missing/late paths,
physical aliases and handle/error semantics, independent regression signals,
CLI/schema compatibility, module size and workflow/docs. Short self-review
finds no remaining actionable issue in this diff. No independent panel.

Existing stable-namespace, output readability and unsupported-backend
fail-closed requirements remain. Early missing/nonregular associated-WAV
rejection is intentional. File identity is not WAV content verification.
No adversarial concurrent namespace lock, atomic multi-file publication,
power-loss or Windows Audio/filesystem/device guarantee. Native Windows
sidecar CI does not cover Windows audio/filesystem identity execution.
No Core/Session/replay/App/runtime/DSP/threshold/schema or frozen Stage-A change;
no musical/human/source/diversity/hardness/release pass, real Development,
Holdout or commercial audio, source-directory discovery, DAW or playback.

## Validation and remaining gates

- Input regression: `/tmp/riotbox-1538-source-safety-{red,green,final}.log`.
- Output self-alias cycle: `/tmp/riotbox-1538-output-alias-{red,green}.log`.
- Associated-WAV red proof: `/tmp/riotbox-1538-associated-input-red.log`.
- Focused tests: `/tmp/riotbox-1538-focused-{debug,release}.log`, actual exit zero.
- Fresh comparator build: `/tmp/riotbox-1538-build.log`.
- Rustfmt, diff/include checks, both manifests and RBX-408 targeted readback pass.
- Full source-free CI: `/tmp/riotbox-1538-ci-final.log`, actual exit zero;
  final warning/error scan clear across CI, both focused runs and fresh build.
- Exact-head native review/merge, archive and cleanup remain required.
