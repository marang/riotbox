# RIOTBOX-1512 — Semantic JamApp lane-to-audio projections

Date: 2026-10-01. Baseline: `0ff4d5e3db71945b8e3cef29aea9ba6201bc2b48`;
the synchronized `0c92d981` adds unrelated Runtime-test ownership and its docs.
Classification: maintenance/regression, not audible instrument progress.

## Ownership and authority

Replace the two mixed textual projection includes with ordinary private
TR-909, MC-202, scene-context, W-30 preview/material/resample modules. Retain
the existing source-phrase mapping child. The projection facade re-exports
exactly the six existing entrypoints with their existing JamApp visibility.
Implementations use `pub(in crate::jam_app)` only where that compatibility
requires it; sibling helpers are `pub(super)` and all others private.

Shared scene selection is independent of lane mapping. W-30 transform data
and defaults live with cached material preparation, not preview policy, so
sample preparation has no dependency back on the preview owner. The seven
private child owners have no dependency cycle and remain below 500 lines
(largest 339). The original inline chop regressions stay beside their private
algorithm and import only that helper instead of a glob.

The facade projects existing Core/Session/action/transport truth; it does not
add another audio runtime, persistent model, queue, scheduler or product policy.
No actual audio loading, hashing, source discovery or callback work is added.
Source-derived decisions, trusted timing, source/section/capture identity,
committed-action selection, scene precedence, missing-material silence and
all sample/preparation rules remain unchanged.

## Retention and tests

- All **63** selected production definitions remain: 59 functions, two
  constants, one struct and one impl. **56** match token-for-token after only
  visibility/trailing-comma normalization, keeping qualified paths and literals.
  Seven complete definitions were separately inspected for Rustfmt-only
  match-arm expression blocks: `build_w30_preview_render_state`,
  `w30_pad_playback_transform`, `mc202_source_phrase_bass_weight`,
  `mc202_source_phrase_stab_bite`, `role_pressure_bias`, `role_accent_mask` and
  `role_destructive_mask`. Their expressions, operands, precedence, order and
  literals are unchanged. A punctuation-only comparison supports, but does not
  replace, those inspections or claim general semantic equivalence.
- All six existing facade signatures match, including parameter/return types,
  lifetimes and their JamApp-only visibility. The existing private source-phrase
  entrypoint retains its full definition and original projection-only visibility.
  Only the moved W-30 transform's three fields gain the sibling visibility
  needed by existing preview preparation.
- Both complete inline chop regressions and their attributes match after the
  single exact glob-to-helper import replacement. No test/assertion/value,
  fixture, cfg or ignored status is removed or weakened.
- App baseline/post-migration execute the identical **771** leaf-name/status
  entries: **770 pass** and one unchanged ignored test. Logs:
  `/tmp/riotbox-1512-baseline-app.log`, `/tmp/riotbox-1512-app.log`.
  A final post-import-fix App run retains the same result and identical entries:
  `/tmp/riotbox-1512-app-final.log`.
- The initial successful all-target compile still contained one unused
  `scene_context` import in TR-909: its local variable shares that name. The
  earlier preliminary warning-free claim was incorrect and corrected in
  Linear/chat. Removing only the unused import produces an explicitly scanned,
  warning-free replacement check: `/tmp/riotbox-1512-check-2.log`.
- The include guard passes at **56 sites / 8 owners**, down from 58 / 9,
  without increasing any allowance. Formatting and whitespace checks pass.
- Full source-free `just ci` passes: App 770, Audio 279, Core 470 and Sidecar
  24 library cases, binary/subprocess tests, Python/contracts, synthetic
  audio/observer smokes and strict all-target/all-feature Clippy. Log:
  `/tmp/riotbox-1512-ci.log`. Exact-head native PR checks separately gate merge.

## Sequential solo review

Applied code-review/Rust lenses to maintainer compatibility, Core/Session
authority, scene precedence, fail-closed source/capture availability, explicit
imports/visibility, sample-preparation bounds and complete test retention.
Reviewed the six lifecycle/capture/test consumer paths, all selected production
definitions and the exact inline test import change. No renderer, timing trust,
capture hydration/identity policy, threshold or fallback is changed. This slice
does not infer real-device behavior from the existing offline/synthetic tests.

The unused-import finding was fixed narrowly and checked explicitly rather than
accepting compiler exit success as warning evidence. No unresolved correctness,
architecture drift, missing-test or scope finding remains; follow-up self-review
has zero additional findings. This is sequential solo review, not independent
panel approval, audio qualification or human listening.

RBX-389 and module policy record actual ownership, not new musical policy.
Frozen Stage-A contracts/algorithms remain untouched. No real source, holdout,
commercial reference, device, DAW or playback is accessed or claimed. This
material production ownership migration is the third substantive architecture
slice since RIOTBOX-1507; test-only RIOTBOX-1511 and archives are excluded.
