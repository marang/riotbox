# RIOTBOX-1543 Feral pack orchestration ownership

Date: 2026-10-01
Implementation baseline: `1d0e8444b0e79d7e99e812eb2b1d141d012f8a9b`.
Integrated archive-only predecessor: `9f39069a6cf4f959bc3ddf8d56f2b08a7e725780`.
Decision: RBX-412
Classification: mechanical maintenance with explicit module ownership.

Feral pack orchestration no longer enters the binary through textual inclusion.
The root owns main, pack_builder owns render_pack and source-format admission,
and pack_report owns the unchanged offline QA value. This removes a P023
artifact-review ownership obstacle without changing a musical mechanism.

## Ownership and compatibility

The root is 280 lines, renderer owner 262 and report owner 54. Imports name
actual type owners explicitly. Only the six helpers still owned by legacy
includes cross a documented transitional parent seam: grid-length validation,
W-30 source-chop rendering, report validation, audio/metrics writing, Markdown
report writing and README writing. Existing root aliases remain only where
untouched legacy consumers need them; test-only aliases use cfg(test).

PackReport retains all 31 field names, types, order and derives, with binary-
local pub(super) visibility. It is an ephemeral QA result, not a second
Core/Session/replay model. Separating it from the renderer prevents a report-
writer/renderer ownership cycle. No new dependency or library API is created.

Baseline/current token inventories match for main, PackReport, render_pack and
validate_source_format, ignoring only comments, whitespace and visibility.
Actual imported type origins are checked separately. The render-to-write-to-
report order, source/output preflights, capacity, error precedence, report
values and source-format checks are unchanged. All shared safety helpers,
output plans, legacy formatter/stem/test bodies and existing tests are intact.

The include guard now counts four sites: three root legacy owners (text output,
stem rendering and tests) and the existing test helper. Policy and inventory
name those remaining owners explicitly. This is not a completed root migration
or a thin-facade claim; the next text-output owner is a separate bounded slice.

## Fresh proof

All 143 focused names/statuses match the pre-edit baseline and pass in Debug
and Release: 95 binary units, six capacity/duration/verification cases and 42
source/comparison/mutual-output safety cases. No test is removed or weakened.
A fresh five-binary build precedes completed sequential CLI captures and hash
checks. All 141 records and 85 artifact hashes match the retained baselines:
Feral 31/39, W-30 32/9, Before/After 31/29 and comparator 47/8.

The original Feral physical-output collision loop rejects against the freshly
built candidate while preserving source and both previous outputs. Both
retained comparison manifests validate. Full source-free just ci finishes
with exit zero; final check, focused, build and CI logs have no warning/error.
Formatting, diff and textual-include guards pass.

Baseline tests: `/tmp/riotbox-1543-baseline-debug.log`.
Final check: `/tmp/riotbox-1543-review-check.log`.
Focused proof: `/tmp/riotbox-1543-focused-{debug,release}.log`.
Fresh binaries: `/tmp/riotbox-1543-build-reviewed.log`.
Full source-free CI: `/tmp/riotbox-1543-ci-final.log`, actual exit zero.

## Review and limits

Sequential solo code-review/Rust/design/spec/evidence lenses inspect the full
changed functions, imported types, retained root consumers, writer call order,
visibility, ownership cycle, module growth and compatibility proof. A short
self-review finds no remaining actionable finding in this diff. No independent
reviewer panel is claimed. Native exact-head PR CI/review, merge/main sync and
archive/cleanup remain separate obligations, not inferred from local proof.

Initial compiler warnings from moved root imports are resolved by removing
only unused aliases or marking genuinely test-only imports; none is suppressed.
Only generated controls and retained fixtures are used. No real Development/
Holdout/commercial audio, source-directory discovery, DAW/device/playback or
subagents. No Core/App/Session/replay/runtime/DSP/algorithm/threshold/schema/
frozen Stage-A change, product fallback or source/musical/hardness/human/release
verdict. Stable-namespace guard limits remain unchanged; this migration adds
no hostile-race, atomic-pack, power-loss or Windows Audio/filesystem guarantee.
