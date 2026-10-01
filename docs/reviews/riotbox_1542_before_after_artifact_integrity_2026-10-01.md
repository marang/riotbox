# RIOTBOX-1542 Before After artifact integrity

Date: 2026-10-01
Implementation baseline: `e6b71be438817077d5f3b4aca151565e21d244bc`.
Integrated archive-only predecessor: `195b30597b72af25fe4966ceb87c110052a2c89c`.
Decision: RBX-411
Classification: maintenance/regression with necessary shared-owner deepening.

Before/After now rejects physical coupling among all fourteen output roles
before even the early source-excerpt write. The four actual QA consumers share
one mutual-output iteration owner, while layouts and identity/error semantics
stay with their existing owners. This removes a P023 artifact-trust blocker;
it changes no musical behavior or qualification claim.

## Independent reproduction and public regression

Two actual CLI probes return zero with identical last-written TR-909 bytes
under hardlinked W-30/TR-909 names. Separate outputs produce distinct stems;
the exact generated input hash stays intact. This independently establishes
output-role coupling, rather than inferring it from the Feral fix.

The first public CLI regression fails on the replaced source excerpt, then
passes with the minimal local guard. Five expanded tests cover all 91 literal
fourteen-choose-two hardlink pairs, six Unix symlink cases in both directions,
a relative stem link through an aliased output directory, late README/manifest
coupling with no early publication, and independent equal-content success.
These are 100 actual CLI invocations, not private identity mocks. The positive
control decodes all six WAVs, verifies distinct stems and expected frames.
Its original composite-duration assumption is corrected against the unchanged
0.75s separator: 0.1s before plus pause plus 0.1s after equals 41895 frames.
No renderer is changed to satisfy that mistaken test expectation.

The original loop against the freshly built exact candidate now rejects with
unchanged input and both prior outputs, while separate mode still succeeds:
`bash /tmp/riotbox-before-after-output-alias-probe.AgQEFj/repro.sh alias /tmp/riotbox-1507-review.4spOix/target/debug/feral_before_after_pack`.
Each invocation owns a fresh directory and one exact generated synthetic input.
The retained Feral and W-30 original collision probes also pass against this
same freshly built candidate, preserving their previous outputs and source.

## One shared safety owner

The Before/After typed plan is 64 lines. One private artifact iterator supplies
both source and output preservation over six WAVs, five metrics and three
metadata files; the composite still has no metrics file. Source-format and
insufficient-window precedence stay first. One renderer integration line adds
mutual preflight before source_samples copying and the early excerpt write.

Review extracts repeated pair classification/iteration into the existing
binary-only qa_source_safety owner, now 64 lines. Feral19, Before/After14,
W-30 pair and comparator pair consume the same helper. The actual path plans
remain local; no new catalog/backend/framework, library interface, product
state, dependency or executable is introduced. Original source identity and
absent/regular helpers remain unchanged. Admit all destinations first, then
delegate existing pairs to that read-only identity guard, with at most 171/91
comparisons for the bounded packs. Zero/one existing output opens no pair
identity handle, preserving explicit no-source W-30 admission.

The comparator retains its lexical self-name rejection and all four referenced
input checks; only its already-existing final output-pair block delegates to
the shared owner. An intermediate unused-function warning is resolved through
that actual consumer, not suppressed. All 37 prior source/comparison/W-30/Feral
safety test bodies remain unchanged; no test is removed, ignored or weakened.

## Fresh proof and review

All 143 focused names/statuses match and pass in Debug and Release: 95 binary
units, six capacity/duration/verification cases, 37 prior safety cases and five
new tests. Fresh five-binary build precedes completed sequential CLI captures
and their hash checks. All 141 records and 85 hashes match the retained baselines:
Feral 31/39, W-30 32/9, Before/After 31/29 and comparator 47/8. Both retained
comparison manifests validate. Full source-free CI exits zero and final logs
contain no warning/error. Fmt, diff, include and targeted RBX-411 gates pass.

Sequential code-review/Rust/design/spec/evidence lenses inspect complete changed
functions, actual callers/writers, distinct-name assumptions, empty/singleton
and late-error behavior, handle lifetimes, bounded offline costs, test
independence, byte compatibility and module ownership. Short self-review finds
no remaining actionable finding in this diff; no independent reviewer panel.

Red/green/final tests: `/tmp/riotbox-1542-output-safety-{red,green,final}.log`.
Focused proof: `/tmp/riotbox-1542-focused-{debug,release}.log`.
Fresh binaries: `/tmp/riotbox-1542-build-reviewed.log`.
Full source-free CI: `/tmp/riotbox-1542-ci-final.log`, actual exit zero.
Native exact-head PR review/CI, merge/main sync and archive/cleanup remain
separate obligations and are not inferred from local success.

Stable namespace only; directories may be created before rejection. No hostile
concurrent namespace locking, atomic pack/power-loss or Windows Audio/device/
filesystem execution guarantee. No Core/Session/replay/runtime/DSP/algorithm/
threshold/schema/frozen Stage-A change, product fallback or source/musical/
hardness/human/release verdict. Only generated controls and retained fixtures;
no real Development/Holdout/commercial audio, source-directory discovery,
DAW/device/playback or subagents.
