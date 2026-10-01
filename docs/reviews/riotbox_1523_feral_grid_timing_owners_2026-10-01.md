# RIOTBOX-1523 — bounded Feral-grid timing ownership

Date: 2026-10-01. Classification: maintenance/regression. Code baseline
`827b0256ea548d520d19081c9cb299379b99a46d` (RIOTBOX-1522 verified feature
head); merged parent `d2d0a5d42ccc2e30c91945706e8af817a50ba63e` integrated
before implementation. This is not a whole Feral-grid root migration.

## Semantic ownership and interfaces

Seven ordinary binary-private production owners expose existing responsibilities:
complete CLI arguments/parsers/help; conservative BPM decision; source-policy
profile; pure Core anchor/groove evidence adaptation; Core probe/readiness
adaptation; bounded groove consumer; readiness serialization/status presentation.
An ordinary adjacent BPM regression file names actual owners explicitly.

The dependency DAG is acyclic: Args -> config; BPM -> Args/config; evidence and
profile are leaves over Core; analysis -> evidence/profile/config;
groove -> BPM/evidence/config; presentation -> BPM/evidence/profile. No new
owner imports the root compatibility facade, orchestration or manifest owner.
The existing complete implementations remain, not speculative adapters or a
second Feral source/timing truth. Core owns source timing. Shared fields/functions
are `pub(super)` within this binary only; single-owner scalar/parsing helpers,
groove constants, anchor and serialization internals remain private.

Root explicit imports preserve untouched lexical consumers, including the
existing manifest and product-stem modules. Test-only compatibility imports
remain conditional. Canonical module policy/inventory/allowlist and RBX-397
record **26 includes/two owners -> 21/two**. Twenty root includes and the one
legacy shared-test-helper include remain counted. Cargo target/dependencies,
library/runtime/DSP, Core/Session/action/replay and frozen Stage-A unchanged.

## Complete behavior proof

- **58 complete definitions** match before/after Rustfmt after only
  visibility/trailing-comma normalization: **42** moved production/helper
  definitions, six retained orchestration definitions and ten complete BPM
  regression/fixture definitions. Whole impls and literals compared, not names
  or counts alone; no expression/algorithm exception was needed.
- **47 executed unit/integration leaf/status entries** remain identical and
  pass in Debug and actual Release: 46 unit tests plus the RIOTBOX-1522 actual
  synthetic CLI integration. No path substitutions, ignores or removed tests.
- **31 actual CLI cases** retain stdout/stderr/status byte identity across help,
  missing/unknown controls, NaN/infinite/negative/text and bar validation,
  missing WAV and successful auto/explicit-140-BPM two-bar packs.
- At the same exact fresh temporary paths under
  `/tmp/riotbox-1523-byteproof.uYJSWo`, both packs retain every **38 artifact
  hash**, including **16 WAVs**, README, reports and manifests. The exact
  newly generated four-second input remains identical too (**39 total**).
  RIOTBOX-1522's quoted `verification_command` is preserved byte-exactly,
  rather than reintroducing the unsafe grammar or ignoring metadata differences.
  Only these named generated control/pack files are accessed or enumerated.
- Baseline and final manifests independently validate their envelope and
  artifact existence. This is metadata proof; separate hashes prove audio and
  report bytes. No waveform pass or human judgment follows from JSON alone.
- Final check/build/Debug/Release logs explicitly warning/error-free:
  `/tmp/riotbox-1523-check-final.log`, debug-final, release, build and baseline
  logs with the same prefix. Initial check/debug failures are retained: a missing
  root drift-label compatibility import and unused root imports were corrected
  without suppressions or changes to any test/algorithm. Draft serializer/anchor
  field visibility was reduced to private before final proof.
- Full source-free `just ci`: **passes** with verified successful exit and final
  warning/error-free scan in `/tmp/riotbox-1523-ci.log`: workspace Rust/Python/
  contracts, synthetic audio gates, formatting/tracked JSON and strict
  all-target/all-feature Clippy. Native exact-head checks, merge/main/archive/
  cleanup remain separate obligations.

## Review and limits

Solo sequential Maintainer, Product, Spec/Evidence, Adversarial, Risk, Rust,
architecture and workflow/docs review, followed by short self-review; not an
independent panel. Complete original functions, import/privacy/DAG changes,
Args error/help order, trust/confirmation branches, labels/clamps, preserved
safe shell grammar, test identities and exact CLI/hash evidence inspected.
**Zero additional changed-diff findings** after import/privacy corrections.
No whole-repository/all-lines or full 8,000-line root audit claimed.

No new audible mechanism, hardcoded musical fallback or source-general claim.
Only explicit freshly generated synthetic controls; no real Development/Holdout/
commercial audio, source-directory discovery, device, DAW/playback or new or
transferred human/music/hardness/release/live-device qualification. Technical
pack reruns do not need duplicate human listening. Maintenance is not musical
instrument progress. RIOTBOX-1509 stays open. Structural cadence advances four
to five since RIOTBOX-1516 only on merge; an architecture checkpoint must precede
another structural slice.
