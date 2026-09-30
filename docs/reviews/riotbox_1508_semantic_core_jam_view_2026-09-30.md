# RIOTBOX-1508 — Semantic Core Jam projections

Date: 2026-09-30
Baseline: `a7195dcb6f95cfefe0017c3e0ba092af8789d8a1` (Core code identical to
the branch start `354762f7`). Classification: maintenance/regression.

## Ownership and scope

Replace eight Jam projection includes and four projection-test includes with
ordinary Rust modules. `view::jam` keeps its public type/function paths through
explicit compatibility re-exports. Private owners separate the assembled model
and builder, source summary/feral evidence, source timing, source map,
scene-launch/transition projections, arrangement readiness, capture summaries
and performer lane/action/Ghost presentation. Types previously appended to
capture/scene shards move to their actual projection owner, not another shared
include namespace. Source-map rows/tests and timing-summary tests lose their
path-attributed hybrid structure. Three regression families share a dedicated
synthetic graph/session/queue fixture owner.

These are existing derived views, not new persistent product models. No public
API, field, label, key, layout, observer schema, ActionCommand, Session/replay
state, source-timing policy, musical threshold, dependency, audio operation or
fallback is changed. All production and shared fixture owners remain below
700 lines; there is no numbered-shard or cosmetic file-budget split.

## Compatibility evidence

- Core library baseline and post-migration tests: **470 passed**, identical
  executed leaf-name multiset. The module-qualified names change naturally;
  assertions and synthetic scenarios remain. Logs:
  `/tmp/riotbox-1508-baseline-core.log`,
  `/tmp/riotbox-1508-core-tests-final.log`.
- All **203** top-level definitions are accounted for: 142 functions,
  23 structs, 15 enums, 19 impl blocks, three constants and one private
  Session accessor trait. After only visibility/trailing-comma normalization,
  **198** match token-for-token, including qualified body paths and literal
  values. The remaining five were individually inspected: Rustfmt adds
  expression blocks around one closure each in source-map navigation, scene
  energy selection, the Jam builder and primary timing-hypothesis selection,
  and around the analyzer-locked readiness match arm. No expression, operand,
  literal or evaluation order changes. This is bounded mechanical evidence,
  not a claim of general semantic equivalence or live audio validation.
- The fixture-backed scene test stays at the same file path and uses the same
  `scene_regression.json`; its `include_str!` literal is unchanged. Public
  projection types and their field/variant shapes retain their existing facade.
  All **35 public struct/enum definitions** match including original field
  visibility, after only whitespace and trailing-comma normalization.
- The all-target Core/App check is warning-free after removal of one
  unnecessary helper import. The shared fixture type/method now has only
  test-parent visibility; its graph/session/queue fields remain private.
  Logs: `/tmp/riotbox-1508-check-final.log`.
- Include guard records **82 sites / 11 owners**, down from 94 / 13, with no
  new include allowance. Formatting and `git diff --check` pass.
- Full source-free `just ci` passes: App 770, Audio 279, Core 470 and Sidecar
  24 library tests, binary/subprocess tests, synthetic smokes, Python/contract
  fixtures and strict all-target/all-feature Clippy. Log:
  `/tmp/riotbox-1508-ci.log`. Native checks at the exact PR head still gate merge.

## Sequential solo branch review

Code-review and Rust-specific lenses cover complete item retention, public
re-exports, field/variant shape, inherent method compatibility, trait scope,
private child/sibling access, synthetic fixture identity, observer/app
consumers, product-spine ownership and contract/workflow evidence. This is one
agent's sequential review, not independent reviewer approval.

Migration-time fixture visibility and an unused import were found by the
compiler, fixed narrowly, and verified by the successful all-target check and
470 Core regressions. No unresolved correctness, architecture drift,
missing-test, unsafe/concurrency or scope finding was identified. Follow-up
self-review likewise found no new issue. Local full CI is green; exact-head
native checks remain required before merge.

No real source, holdout, commercial reference, playback, audio device, DAW or
new human/musical verdict was accessed or claimed. Frozen Stage-A contracts
remain untouched. This semantic ownership migration is the first substantive
slice after the merged/closed RIOTBOX-1507 architecture checkpoint; the separate
RIOTBOX-1340 import-only cleanup does not advance that cadence.
