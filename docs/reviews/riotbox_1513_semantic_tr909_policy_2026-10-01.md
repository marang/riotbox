# RIOTBOX-1513 — Explicit Core TR-909 policy ownership

Date: 2026-10-01. Baseline: `6d4942db1b8fce77c31e76265611b60208e756f3`;
the later RIOTBOX-1512 archive synchronization changes only closeout docs.
Classification: maintenance/regression, not audible instrument progress.

## Ownership and compatibility

Replace both `tr909_policy` textual includes with an explicit public facade
over private typed model, render composition, source-support and pattern/phrase
owners. Core retains the existing policy authority; Source Graph, Session and
transport remain inputs, and App/Audio remain consumers. There is no new crate
dependency, product state, policy, scheduler or generated fallback.

The facade explicitly re-exports all nine existing public types and three
public functions. Shared internal support data and the three cross-owner
helpers are bounded to the policy subtree; remaining helpers stay private.
Ordinary synthetic regressions consume the facade through explicit imports.
All selected owners remain below 500 lines; the largest is the cohesive
425-line regression family. No numbered shards or wildcard imports remain.

The inventory's old RIOTBOX-1331 association was stale. Its exact archived
scope shipped source-derived lane differentiation in PR #1304; it did not
remove these includes. RIOTBOX-1513 owns only the current structural migration,
not another musical implementation or a retroactive change to that evidence.

## Retention and verification

- All **29 production definitions** match after only visibility/trailing-comma
  normalization: eleven functions, eight enums, two structs and eight impls.
  Complete algorithm bodies, literals, ordering and qualified paths remain.
- All **17 complete model/impl definitions** also match under the stricter
  comparison that retains visibility, public fields/variants, method visibility
  and attributes. All three public function signatures and attributes remain
  compatible through explicit root exports. No public name/path is removed.
- Only the existing private support struct's two fields and three existing
  helper functions gain the sibling visibility needed by their new consumers.
  That visibility is policy-parent-only, not crate-wide or public.
- All **ten test/helper definitions**, including **five regressions**, retain
  complete bodies, attributes, assertions and data after trailing-comma
  normalization. The former outer `#[cfg(test)] mod tests` moves to the facade's
  ordinary child declaration; the same test file and exact `include_str!` path
  still resolve the same committed JSON projection fixture. No fixture content
  or actual audio is opened for this comparison.
- Core baseline/final libraries retain identical **470** executed leaf/status
  entries, all passing. Logs: `/tmp/riotbox-1513-baseline-core.log`,
  `/tmp/riotbox-1513-core-final.log`.
- Initial all-target/test compilation found one omitted test enum import.
  Added only `Tr909PhraseVariationPolicy` at that consumer. The replacement
  Core/App/Audio all-target check succeeds, with the log explicitly scanned
  for warnings/errors: `/tmp/riotbox-1513-check-2.log`. No glob, suppression,
  assertion change or visibility broadening masked the compile finding.
- Include guard passes at **54 sites / 7 owners**, down from 56 / 8, without
  an allowance increase. Formatting and whitespace checks pass.
- Full source-free `just ci` passes, including strict all-target/all-feature
  Clippy, metadata contracts and synthetic smokes. Integrated library results:
  Core 470 pass, App 770 pass / one unchanged ignore, Audio 279 pass / one
  unchanged ignore, Sidecar 24 pass. Log: `/tmp/riotbox-1513-ci.log`.
  Exact-head native PR checks remain a separate merge gate.

## Sequential solo review

Applied code-review/Rust lenses to public compatibility, semantic boundaries,
Core authority, mode/routing composition, source/scene/transport precedence,
pattern/phrase adaptation, private helper visibility and test/fixture retention.
Inspected all original production definitions and the complete regression
family; normalized comparisons constrain each moved implementation, and
explicit import review checks actual owner resolution. Public label/method and
typed projection contracts remain intact. This is sequential solo review, not
an independent reviewer panel or fresh musical qualification.

No unresolved correctness, architecture drift, missing-test or scope finding
remains after the narrow compile fix and replacement checks. Follow-up
self-review identifies zero additional findings. Native exact-head CI still
gates merge; local synthetic evidence is not live-device or listening proof.

RBX-390 and module policy record ownership only. No source-support algorithm,
threshold, policy precedence, Session/replay/schema, DSP or frozen Stage-A
contract changes. No real source, holdout, commercial reference, device, DAW,
playback or musical/human verdict is accessed or claimed. This material policy
ownership slice is the fourth substantive architecture slice since RIOTBOX-1507
when merged; test-only and archive work remain excluded.
