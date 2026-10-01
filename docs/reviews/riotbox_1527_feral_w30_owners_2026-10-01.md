# RIOTBOX-1527 — Feral W-30 QA ownership migration

Date: 2026-10-01
Decision: RBX-398
Classification: bounded, behavior-preserving Rust module migration
Baseline: `a7e1f504aca096ce04ce2427021d0c35a6ef9f68`
Integrated predecessor: `09cbb61b05aa3c76e8603f9d36e10244acf26c16`

## Outcome and scope

Four W-30 policy/playback includes and the W-30 regression include become
ordinary binary-private modules. The former chop file mixed window preparation,
scalar measurements, trigger policy and JSON presentation. A one-for-one
conversion would preserve those responsibilities and introduce an accent/trigger
dependency cycle. Existing measurements and trigger-event data are dependency
leaves; serializers consume policy evidence without governing it.

This follows the RIOTBOX-1524 architecture checkpoint. It removes an explicit
maintainability blocker alongside P023, not an audible Golden Path blocker or a
product mechanism. The remaining Feral root is still legacy, not fully migrated.

| Owner | Responsibility and dependencies |
| --- | --- |
| `sample_measurements` | Unchanged mono/RMS/peak/delta helpers; no orchestration dependency |
| `w30_source_events` | Existing trigger-event data, no policy or serialization dependency |
| `w30_source_chop` | Window selection, gain/articulation and loop closure; measurements/config |
| `w30_slice_choice` | Candidate scoring and private offsets; measurements |
| `w30_source_accent_dynamics` | Source-energy accents and evidence; events/measurements/slice count |
| `w30_source_trigger_policy` | Beat-anchor plan and quantization evidence; Grid/slice/accent/events |
| `w30_source_playback_profile` | Existing playback profile; articulation/spectral metrics |
| `w30_source_manifest` | Five unchanged DTOs and conversions; policy evidence/config |

All modules import actual owners, not root compatibility imports. Cross-owner
items are `pub(super)` only. Selected offsets, candidates, local helpers,
loop/quantization thresholds and serializer fields remain private. Explicit root
imports preserve unconverted consumers. Ordinary W-30 tests import actual owners;
their render calls still use the untouched legacy stem renderer at the root.
No mock, adapter trait, new state/model, dependency or public API is introduced.

## Preservation evidence

Before edits, both actual Cargo profiles passed 52 unit and two integration
regressions. After migration, all 54 executed leaf names/statuses remain exact
and pass in Debug and Release, without path substitutions or removed/ignored
tests. The seven W-30 regression functions and three synthetic fixtures retain
their complete bodies. The owning behavior-preserving migration contract takes
precedence over generic test-replacement guidance.

All 63 complete definitions remain equal after Rustfmt with only visibility and
trailing-comma normalization: 53 production/helper/type/constant/complete-impl
definitions plus ten regression/fixture definitions. This compares whole items,
not only function fragments or output fingerprints. No algorithm, numeric or
string literal, threshold, trust condition, label, field/schema, envelope,
offset, gain or frame calculation changes.

The standalone binary was built before each CLI comparison. Thirty-one actual
CLI cases retain exact exit status, stdout and stderr: help/parser/errors and
two successful auto/explicit-140-BPM two-bar packs. Both pack manifests validate
artifact existence. All 39 hashes remain exact: 38 output files, including 16
WAVs, reports, README and manifests, plus the unchanged input WAV. Thus prior
literal verification syntax and the current numerical evidence remain intact.

The sole accessed input is the exact named fresh four-second synthetic control
from the existing generator:
`/tmp/riotbox-1527-byteproof.VAH6vh/synthetic-control.wav`.
Re-rendering intentionally overwrites only this run's bounded synthetic outputs.
No source directories or real Development/Holdout/commercial audio are accessed.

RIOTBOX-1525's mechanical Grid capacity guard and RIOTBOX-1522's literal POSIX
verification grammar remain unchanged. RIOTBOX-1526 was merged and fast-forwarded
before edits; its standalone W-30 changes do not overlap this Feral ownership
slice. Access, error, directory/artifact publication and numeric-gate ordering
remain the existing behavior, not a newly transactional QA pipeline.

## Review and verification

Solo sequential correctness, Rust/architecture, test/spec and workflow lenses,
followed by self-review; no independent reviewer panel is claimed. No actionable
new finding remains. Dependency direction, privacy, whole-impl retention,
unchanged renderer/runtime boundaries and prior fixes were inspected. The
largest changed owner is the 282-line regression module; production owners are
at most 221 lines. No warning suppression or mechanical numbered shard is added.

The include guard passes with 21 to 16 sites in the same two owners: 15 at the
legacy root and one shared legacy test helper. The allowlist shrinks; no new
allowance is granted. Canonical module policy, inventory and decision log agree.

Commands and bounded logs:

- `cargo check -p riotbox-audio --bin feral_grid_pack --tests`:
  `/tmp/riotbox-1527-check.log`
- Debug/Release focused tests:
  `/tmp/riotbox-1527-{baseline,final}-{debug,release}.log`
- Baseline/final standalone builds:
  `/tmp/riotbox-1527-{baseline,final}-build.log`
- `scripts/check_no_textual_includes.sh` and `git diff --check`
- Full source-free `just ci` passed: `/tmp/riotbox-1527-ci.log`.
  The finalized documentation snapshot is rechecked before PR; exact-head
  native Rust/Windows Sidecar jobs remain mandatory merge gates.

Final focused check/build/test logs are explicitly warning/error-free. No real
device, DAW, human playback, listening verdict or fresh source qualification was
performed. Fixed QA patterns and source-derived metrics here are technical
controls, never source-general/product/music/hardness/release/live-device proof.
Core/Session/action/replay, library runtime/DSP and frozen Stage-A are untouched.

## Closeout obligations

After PR review and both native jobs pass, merge only the reviewed full head,
sync main, archive the Linear issue, merge/validate that archive and clean only
its exact branches. Structural cadence advances from zero since RIOTBOX-1524 to
one only on this implementation merge. RIOTBOX-1509 remains Todo awaiting a
fresh causal Windows startup reproduction, not closed by unrelated green CI.
