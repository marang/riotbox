# RIOTBOX-1530 — Feral MC-202 QA policy and evidence owners

Date: 2026-10-01
Baseline: `e29edd82f7a2540af55311f0f2250ebce6c3c7a2`
Integrated predecessor archive: `f9c60b6ab7c05e7890d677bcf6836a93fb9298a9`
Decision: RBX-401
Classification: maintenance/regression; behavior-preserving semantic migration

## Outcome and scope

The existing MC-202 QA contour, phrase/state policy, pressure rendering/proofs,
low-body DSP and JSON presentation now have ordinary binary-private owners.
Contour classification and low-dominance computation consume actual Grid,
config and numerical measurements. Phrase/state policy consumes contour and
the existing TR-909 support profile. Pressure consumes those policies, low-body
DSP, shared render measurements and the existing library renderer. Presentation
consumes the typed proofs, never the reverse.

Whole existing RenderMetrics, render_metrics and rms_delta move from pack,
stem and mix orchestration into a numerical dependency owner. Without that
extraction, a direct include conversion would keep policy depending backwards
on the root. These are in-process computations: no new adapter, mock, public
library interface, product renderer or Session/source/replay/arrangement truth.
Typed Mc202PatternOrigin and all complete impls remain intact.

Shared fields/functions and retained assertion constants are binary-bound.
Single-owner phrase/DSP helpers, thresholds and DTO fields remain private.
Root explicit/cfg-test compatibility imports retain untouched legacy callers,
including existing manifest metric types. Other root families remain legacy;
this is not a completed Feral root migration or an audible Golden Path closure.

This bounded improvement-track A slice supports reviewability beside musical
work under the user's explicit autonomous non-DAW authorization. It introduces
no new musical mechanism and changes no musician action or expected sound.
Codebase-design informs the one-way in-process seams; the repo migration
contract takes precedence over generic advice to replace existing tests.
Domain-modeling records the ownership trade-off in the canonical Decision Log,
not a competing glossary or ADR system.

## Exact preservation

All 90 complete affected definitions remain equal after Rustfmt with only
visibility and trailing-comma normalization: 44 moved and 46 retained
production/helper/type/constant/whole-impl definitions. Equality covers complete
expression plans, classification, pressure reinforcement/envelopes, headroom,
low-body processing, render measurements and retained orchestration. No pattern,
threshold, label/schema, allocation or source-access/error/publication ordering
changes. Existing capacity/literal-command and W-30/TR-909/window-owner fixes
are retained. All existing test/fixture files and bodies remain unchanged.

Both actual Cargo profiles pass before and after edits with the same 52 unit
and two integration names/statuses. No removed, ignored or substituted test.
Actual standalone builds precede baseline/final CLI comparisons. All 31
status/stdout/stderr results remain byte-identical. The bounded auto and explicit
140-BPM two-bar packs retain all 39 hashes: 38 outputs including 16 WAVs,
complete reports/README/manifests, plus the sole input. Both manifests validate
existing artifact paths. Algorithms are not tuned against these results.

The sole file input is the exact fresh generated four-second synthetic control:
`/tmp/riotbox-1530-byteproof.hHxTj3/synthetic-control.wav`.
Existing unit fixtures are synthetic/in-memory. Only this run's bounded
technical output packs are overwritten for comparison. No real Development,
Holdout or commercial audio, source-directory discovery, device, DAW or playback.

## Review and diagnostics

**P3 — incomplete legacy compatibility imports, fixed.**
API and Consumer Compatibility lens: removing root BarVariationMetrics and
SpectralEnergyMetrics aliases breaks existing manifest.rs imports even though
the new policy imports their actual owners. Keep those narrow compatibility
type imports; production/test compilation and all CLI/artifact comparisons
verify that unchanged consumers remain supported. Bar-variation function and
MC-202 note-budget compatibility imports stay test-only where applicable.

The first migration diagnostic also exposed visibility markers incorrectly
placed on multiline function parameters by the initial mechanical transform.
Correct them before validation; only shared struct fields/functions and actual
cross-owner methods carry visibility. Failed initial formatting/check logs are
diagnostic evidence, not green verification. No new warning suppression.

Solo sequential correctness, Rust/architecture, tests/spec and workflow review,
then short self-review: zero remaining actionable findings, no independent
panel. Actual dependency directions, typed origin, privacy and library-render
consumers inspected. Owners follow concepts, not numbered or line-count shards.
The two converted includes shrink inventory 12 → 10 in the same two owners
(nine root, one legacy shared test helper), without allowance expansion.
Policy, inventory and EOF RBX-401 agree.

The Project Updates document reached Linear's update-size limit. Its history is
unchanged; a linked October continuation retains verified merge/cleanup state.
This is an operational rollover, not a product decision or rewritten history.
RIOTBOX-1509 remains Todo awaiting causal Windows startup evidence.

## Verification and remaining gates

- Baseline Debug/Release/build logs: `/tmp/riotbox-1530-baseline-*.log`
- Final reviewed check: `/tmp/riotbox-1530-check-reviewed.log`
- Final Debug/Release/build logs: `/tmp/riotbox-1530-final-*.log`
- Both exact synthetic manifest validators; Rustfmt, diff/include guards;
  final focused logs explicitly scanned warning/error-free
- Full source-free `just ci` before PR; exact-head native Rust and Windows
  Sidecar jobs/review before merge, then archive and exact cleanup

These synthetic metrics/fixed phrases remain diagnostic scaffolds. Preserving
the existing QA SourceDerived label grants no product/source-general, music,
hardness, human, release or live-device proof. No runtime/DSP/library API,
Core/Session/action/replay or frozen Stage-A change. Windows transport CI does
not prove Windows audio/filesystem/device behavior. Structural count advances
three to four since RIOTBOX-1524 only on implementation merge.
