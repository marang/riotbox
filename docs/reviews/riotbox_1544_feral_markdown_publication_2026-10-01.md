# RIOTBOX-1544 Feral Markdown publication ownership

Date: 2026-10-01
Implementation baseline: `5c9081386c47ff34c10232d37726bba9b0869842`.
Integrated archive-only predecessor: `5c741a4a935ffeafdd6787ca80f3c4e873101ce9`.
Decision: RBX-413
Classification: mechanical maintenance with explicit presentation ownership.

Feral report/README publication is an ordinary binary-private module rather
than textual root capture. The renderer imports both file publishers directly
from their sibling owner. This removes a P023 QA artifact-review obstacle;
it changes neither the evidence algorithms nor a musical mechanism.

## Ownership and compatibility

The Markdown owner exposes only write_report and write_readme as pub(super).
All nine formatting helpers remain private. Actual Args/Grid/PackReport,
source-window selection, Core readiness report, BPM decisions/labels,
readiness labels and constants are imported from their existing owners.
Four transitional parent bridges remain in the renderer for legacy stem,
validation and audio/metrics publication; the two Markdown bridges disappear.

The root is 272 lines, renderer 263 and Markdown owner 470. The latter retains
one cohesive publication responsibility rather than sharding its long report
format by line count. No wildcard imports, adapter/framework, new public
library API, timing analysis or Core/Session/replay/product model is introduced.
Only compiler-proven obsolete root readiness imports are removed; manifest,
stem and test consumers retain their compatibility aliases.

All eleven normalized complete function bodies match the pre-edit inventory,
ignoring formatting/comments and binary-private visibility. Literal strings,
precision, field/argument order, values and write/error order stay unchanged.
The entire root main and renderer algorithms, shared report value, safety
helpers/plans, legacy stem/test/helper bodies and existing tests are intact.
The include guard counts three remaining sites: root render_stems/tests and
the test-helper include. Policy/inventory explicitly retain those unfinished
owners. This is not a completed root migration or thin-facade claim.

## Fresh proof

Pre-edit baseline and final Debug/Release each pass the same 143 focused
names/statuses: 95 binary units, six capacity/duration/verification cases and
42 source/comparison/mutual-output safety cases. No test is removed or weakened.
Fresh five-binary build precedes completed sequential CLI captures and hashes.
All 141 CLI records match the retained original baselines; all 85 artifact
hashes match the accepted predecessor's identical original-baseline bytes:
Feral 31/39, W-30 32/9, Before/After 31/29 and comparator 47/8.

The original Feral collision loop rejects against this freshly built candidate
while preserving its generated input and both previous outputs. Both retained
comparison manifests validate with existing artifacts required. Full source-
free just ci exits zero; final check, focused, build and CI logs contain no
warning/error. Formatting, diff, include and targeted RBX-413 gates pass.

Baseline: `/tmp/riotbox-1544-baseline-debug.log`.
Final check: `/tmp/riotbox-1544-review-check.log`.
Focused proof: `/tmp/riotbox-1544-focused-{debug,release}.log`.
Fresh binaries: `/tmp/riotbox-1544-build-reviewed.log`.
Full source-free CI: `/tmp/riotbox-1544-ci-final.log`, actual exit zero.

## Review and limits

Sequential solo code-review/Rust/design/spec/evidence lenses inspect complete
functions, direct imported type/function origins, renderer call sites, private
formatting and publication boundary, root compatibility consumers, module
growth, dependency direction, unchanged test identities and byte proof. Short
self-review finds no remaining actionable finding. No independent panel is
claimed. Native exact-head PR CI/review, merge/main synchronization and archive/
cleanup remain separate obligations, not inferred from local success.

Initial moved-import warnings are resolved without suppression. The existing
broader architecture review is due after this fifth material branch since
RIOTBOX-1539, before another family change; no new global cadence is invented.
Only generated controls and retained fixtures are used. No real Development/
Holdout/commercial audio, source-directory discovery, DAW/device/playback or
subagents; no DSP/algorithm/threshold/schema/Core/App/Session/replay/runtime/
frozen Stage-A change, fallback or source/musical/hardness/human/release verdict.
Stable-namespace guard limits remain unchanged. This adds no hostile-race,
atomic-pack, power-loss or Windows Audio/filesystem/device guarantee.
