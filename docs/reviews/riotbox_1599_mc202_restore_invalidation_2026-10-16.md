# RIOTBOX-1599 — MC-202 restore invalidation and legacy resolution

Date: 2026-10-16. Base: `f30ac128ec92d112d4a11be669196ff0baebb592`
(RIOTBOX-1597 merged). P017 maintenance protecting P023 recall under
RIOTBOX-1041. RBX-457 and the owning replay/Session/module contracts freeze
the missing-plan resolution boundary; RBX-458 separately freezes the demonstrated
SetRole replay-state correction before its implementation.

## Failure and repair

The checkpoint's valid history confirms a Grid, commits an MC-202 plan, performs
a matching Revert and reconfirms the same Source/Grid. Live state, saved Session,
ordinary JSON reload and Core replay retain `None`. App target postprocessing
previously resurrected the old persisted plan because it scanned only phrase
actions and checked the final Source ID, ignoring intervening invalidation.
The unvalidated constructor had the same temporal blind spot.

Core now owns one shared transactional traversal for graph-free and graph-aware
replay, retaining duration/binding checks, scene movement, reports and candidate
installation. Purpose-specific target/payload interfaces may resolve a missing
legacy MC-202 plan at its actual suffix entry. The resolver borrows immutable
pre-entry Session state, including phrase memory, and returns only a plan.
Core captures the ordinary Undo snapshot and executes the normal action before
installing that missing decision. Persisted `Some`, including degraded/stay-out
decisions, bypasses selection. Existing default interfaces remain no-backfill.
Later effective Source/Hypothesis Revert uses the existing Core invalidation;
reconfirmation, timing-only or empty suffixes cannot regenerate skipped history.

App supplies the existing pure selector through a small compatibility adapter,
with the same roles, touch defaults/clamping, mutation intent and pre-entry
memory. No complete Session clone per missing entry or mutating side-effect
wrapper. Restore no longer performs a full-prefix reconstruction after Core;
the existing current Source-ID trust clearing remains explicit. Ordinary JSON
loading does not acquire a re-planning side effect.

The private `restore_replay::legacy_mc202` constructor owner deliberately does
not admit or reorder unvalidated history. It retains prefix clamping,
first-match duplicate masking, accepted/Committed filtering and stored-record
order, now including timing entries through Core's narrow single-entry resolver
execution. One scratch Session supplies only its final plan back to the caller.
No phrase history preserves opaque runtime metadata. Unvalidated persisted
phrase metadata remains recoverable despite malformed phrase params; malformed
timing instead clears the plan and stops. This is neither replay admission nor
source qualification. The retained linear-scan oracle checks join compatibility.

Independent Rust/Adversarial review also found a pre-existing executor/live
discrepancy: SetRole lowered live touch directly, but Core maximized it. A public
live-commit comparison demonstrated subsequent missing plans receiving `.9`
instead of `.2` through both restore paths. RBX-458 corrects Core SetRole to
direct assignment, including ordinary replay. Generate/Mutate retain their max
semantics. This is an explicit narrow default-execution correction, not a source
selector/threshold/recipe change or an App shadow touch history.

## Evidence

Each genuine RED below compiled and executed exactly one regression test:

- Public live→save→ordinary load/Core/App target regression:
  `/tmp/riotbox-1599-plan-invalidation-red.log`, SHA-256
  `3138b7565b1503800540a34d4ec2918ed6bb6e54baea8a9ad9d00f5d610b2544`.
  Repeated on post1597 main before the repair. Writes exactly one generated
  Session JSON, with no source/Graph refs, captures, receipts or hydrated audio.
- Missing-plan payload suffix:
  `/tmp/riotbox-1599-payload-legacy-red-actual.log`, SHA-256
  `c4b79d68e0d1c2983f1423ef940e9b9fa31db7bf6e65062909a2f4117f79ca58`.
- Constructor matching-Revert/reconfirmation:
  `/tmp/riotbox-1599-constructor-red.log`, SHA-256
  `3bd385907ac85c3b8df718b030d1a7e862dfb219e27cea0edba2d07221fea812`.
- Actual live SetRole writer versus both public restore paths:
  `/tmp/riotbox-1599-touch-red.log`, SHA-256
  `268a4eac713a04a0fd6439257b90cf4f381fab182f503151d558ab7835066e38`.
  GenerateFollower(.9)→SetRole(Follower,.2)→GenerateAnswer(.1) gives live
  `(plan.touch, macro)=(.2,.2)` versus pre-fix restore `(.9,.9)`.

All four regressions subsequently passed. Earlier short-name filters selecting
zero tests are excluded from RED evidence; compile-only setup errors are not
semantic REDs. The PCM positive control initially exposed its generated input's
default zero music-bus level, not a product defect; the fixture now explicitly
sets a nonzero bus level before exercising the production projection.

Public App integration suite: **14 passed**, no filtered/ignored tests,
`/tmp/riotbox-1599-app-controls-final.log`, including the final malformed-input
constructor control. Its tables cover **30 target-prefix
cases**, **90 payload anchor/target cases** (anchors0–4, empty suffix included),
and **30 constructor-prefix cases**, not that many separate Rust test functions.
All three declared policies and Graph present/absent paths are included.
Other controls cover mismatched Source/Hypothesis Reverts, rejected/unaccepted/
uncommitted filtering, a genuinely new missing phrase after reconfirmation,
final Source-ID trust clearing and full App Session/queue/allocator/transport/
projection/view nonmutation on late execution error. The constructor separately
retains Undone filtering and the existing adversarial first-match scan oracle.

The exact App MC-202 projection feeds the existing renderer in memory: a
persisted generated plan at prefix2 produces finite nonzero PCM, while the
invalidated prefix4 projects Silent/None and produces exact zero PCM. This is
offline output-path regression proof, not a callback/host observation, source
intelligence, human listening or musical qualification.

Six Core public-interface controls passed in
`/tmp/riotbox-1599-core-traversal-controls.log`: immutable pre-entry plan memory
and ordinary Undo snapshots, persisted source/stay-out bypass, default graph-free
no-backfill, direct/default SetRole lowering/clamping/defaults versus Generate/
Mutate max, full-Session rollback after a callback and late invalid/unsupported
entry, and Core planner Undo/rejected/uncommitted exclusion.

Complete App library: **932 passed, one existing ignored benchmark**, no
failures, `/tmp/riotbox-1599-app-full.log`. Complete Core library: **565 passed,
one existing ignored benchmark**, `/tmp/riotbox-1599-core-lib.log`. Full source-free
local/native CI and exact-head PR/merge evidence remain mandatory; final results
belong to PR/Linear closeout and must not be inferred from these focused runs.

## Review and limits

The retained Rust/Adversarial SetRole P2 is fixed with its genuine public
RED→GREEN and explicit owning decision/spec amendment. Final independent
Rust/Adversarial/Maintainer and Spec/Evidence/Product/Risk reviews retained no
additional P0–P3 finding. Coordinator self-review checked all changed owners,
caller-owned versus transactional boundaries, compatibility, allocations,
selector inputs, test signal and documentation drift: no further finding.
The final malformed-constructor control and report received a short independent
Spec/Evidence review delta with no retained finding. Its interim wording about
an unexecuted control was superseded by the actual final 14-test run above;
no pending CI result is inferred from code inspection.

No new ActionCommand, serialized Session/replay/recording/observer field/schema,
persistent App state, second history/validator/allocator/clock, dependency or
source/DSP algorithm/threshold/recipe. No actual user Session migration,
source/capture/Holdout/commercial/ignored-attempt access or discovery, host/device/
service observation/runtime, consumed-phase repeat, playback, DAW or Windows/TUI
expansion. No human, musical, source-general, hardness, latency, endurance or
release claim. RIOTBOX-1598,1041/1036 and the original ALSA cause remain open;
RIOTBOX-1589's passed and consumed host observation is unchanged.
