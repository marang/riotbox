# RIOTBOX-1510 — Semantic Source Graph timing candidates

Date: 2026-09-30. Baseline: `bf65b983af7c79e088d960b1f446c7f9a802ec0b`;
the later synchronized `33ae0c36` contains only unrelated closeout/report docs.
Classification: maintenance/regression, not audible instrument progress.

## Ownership and scope

Replace all twelve candidate `include!` sites with private semantic modules
behind the existing `source_graph::timing_probe_candidates` interface. Explicit
exports retain public compatibility, including the Source Graph root exports.
Owners separate normalized onset evidence, period scoring, downbeat selection,
hypothesis construction, beat/bar/phrase grids, drift/groove, final model
assembly, confidence/readiness reports and grid-use policy. Existing warning
messages move to model assembly; high-drift classification moves to drift.
Shared evidence no longer lives in hypothesis construction, avoiding a hidden
dependency cycle from scoring/groove back to constructed hypotheses.

Hybrid candidate test files become ordinary regression-family children.
Synthetic input builders have a separate test-only owner. The two comparator
regressions live directly under period scoring so its ordering helper and
report-only score fields remain private. Every helper is private or bounded to
the candidate/test subtree; no new crate-wide helper surface is introduced.
All selected Rust owners remain below 500 lines, without numbered shards.

No algorithm, filtering/sorting, literal, threshold, grid, drift/groove,
warning order, readiness/trust/degraded policy, public path, schema, dependency,
ActionCommand or Session/replay state changes. RIOTBOX-1330's archived scope
was trusted live-ingest/confirmation wiring, not this include migration.

## Compatibility evidence

- All **168** selected definitions are accounted for: 128 functions,
  18 structs, seven enums, five impl blocks and ten constants. **166** match
  token-for-token after only visibility/trailing-comma normalization, retaining
  qualified body paths and literals. The remaining two were individually
  inspected: Rustfmt adds closure expression blocks in `period_score_order`
  and `probe_candidate_groove_residuals`. Their expressions, operands and order
  are unchanged. This is bounded mechanical evidence, not a general semantic
  equivalence or audio-quality claim.
- All **15 public struct/enum definitions**, including field/variant visibility
  and attributes, and **12 public function signatures** match after only
  whitespace/trailing-comma normalization. All five complete impl blocks,
  including inherent method visibility, and all **47** test bodies/attributes
  match under the same stricter comparison. Explicit facade exports preserve
  each of the 27 public names.
- Core baseline/post-migration libraries both pass **470 cases**, with exactly
  the same executed leaf-name multiset. Module-qualified names change; no
  assertion or synthetic scenario is removed. Logs:
  `/tmp/riotbox-1510-baseline-core.log`, `/tmp/riotbox-1510-core.log`.
- App library results retain the same **771** executed leaf-name entries:
  770 passed and one unchanged ignored test. Compared the pre-migration full
  gate `/tmp/riotbox-1508-ci.log` with `/tmp/riotbox-1510-ci.log`; App code is
  unchanged between these baselines.
- All-target Core/App checks are warning-free. A migration-time transitive
  private `DownbeatPhaseScore` interface was corrected to candidate-parent
  visibility, not public or crate-wide visibility. Its fields are consumed by
  model assembly. Comparator/report-only fields remain private. Log:
  `/tmp/riotbox-1510-check-2.log`.
- Include guard passes at **70 sites / 10 owners**, down from 82 / 11, without
  an allowance increase. Formatting and whitespace checks pass.
- Full source-free `just ci` passes: App 770, Audio 279, Core 470 and Sidecar
  24 library cases, binary/subprocess tests, Python/contract fixtures, synthetic
  audio/observer smokes and strict all-target/all-feature Clippy. Log:
  `/tmp/riotbox-1510-ci.log`. Exact-head native PR checks still gate merge.

## Sequential solo branch review

Applied code-review/Rust lenses: maintainer compatibility and data flow;
product scope and semantic ownership; spec/evidence retention; adversarial
empty/nonfinite/sparse onsets, ambiguity, loop-boundary and drift readiness;
and risks of moved private interfaces/tests. Inspected complete definitions,
imports/re-exports, ordinary child resolution, live-ingest consumer wiring and
synthetic fixture identities. No runtime I/O or callback path is added. This
is sequential solo review, not independent panel approval.

The compiler-detected transitive private-interface mistake was fixed narrowly
and verified by the warning-free check and 470 Core cases. No unresolved
correctness, architecture drift, missing-test or scope finding was identified.
Follow-up self-review found zero additional findings. Integrated local CI is
green; exact-head native PR checks remain required before merge.

RBX-387 and module policy record the actual owners, not new timing policy.
No real source, holdout, commercial reference, device, DAW, playback or new
musical/human verdict was accessed or claimed. Frozen Stage-A contracts remain
untouched. This is the second substantive ownership migration after the closed
RIOTBOX-1507 architecture checkpoint; archive/report-only work is excluded.
