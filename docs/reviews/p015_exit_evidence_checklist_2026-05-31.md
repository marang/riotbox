# P015 Exit Evidence Checklist - 2026-05-31

Scope:

- `P015 | Productization Alpha`
- RIOTBOX-1037 and RIOTBOX-1050 through RIOTBOX-1056
- perform-first Jam taste/proof language
- first-run next-move guidance
- Help overlay readability
- P012, P013, and P014 regression baselines

Status:

- exit-evidence checklist drafted
- no runtime behavior changed by this document
- P015 remains open until the phase DoD is updated from this evidence and the
  final phase-level verification has passed

## Productized Surfaces

- RIOTBOX-1037 translated proof-heavy Jam perform and inspect surfaces into
  compact musician-facing taste/proof language while keeping deeper proof detail
  in inspect-oriented surfaces.
- RIOTBOX-1050 added Recipe 16 and an executable proof path so compact perform
  cues, inspect-owned proof detail, and source-backed scene-ready taste language
  stay aligned.
- RIOTBOX-1051 added in-app Help cues for Jam taste/proof states so a musician
  can interpret cautious, scene-ready, and proof language without leaving the
  instrument.
- RIOTBOX-1052 added a durable musician-facing glossary for the P015
  taste/proof vocabulary and linked it from screenshot/docs surfaces.
- RIOTBOX-1053 audited first-run next-move guidance so scene jump is not
  promoted as equally safe when Arrangement / Scene readiness is cautious,
  fallback-backed, or unknown.
- RIOTBOX-1054 fixed Recipe 16 to follow the same timing-trust boundary:
  `y` is the first move only when Jam says `taste scene-ready`; otherwise
  `g` follow or `f` fill remains the safer next musical move.
- RIOTBOX-1055 pointed first-run Help follow-up copy at Recipe 16 so the first
  loop naturally leads to taste/proof reading instead of only generic gesture
  recipes.
- RIOTBOX-1056 widened the Help overlay and added snapshot coverage for dense
  first-run, scene-timing, taste/proof, primary, and advanced Help sections.

## Evidence Gates

Minimum P015 exit evidence should include:

- `cargo fmt --check`
- `git diff --check`
- `just ci`
- `just audio-qa-ci`
- `just p015-jam-taste-recipe-proof`
- `just p014-scene-movement-observer-probe`
- `just p012-all-lane-source-grid-output-proof`
- `just representative-source-showcase-musical-quality` against the current
  representative showcase artifact set

The P015-specific gate is `just p015-jam-taste-recipe-proof`. It proves the
perform/inspect split and Recipe 16 taste/proof path, but it is not a substitute
for the P012/P013/P014 regression gates.

## Product Spine Checks

Before closing P015, verify:

- no new ActionCommand was added without queue, commit, Session/replay,
  observer/user surface, and QA accounting
- P015 copy still derives from shared Jam projection/model surfaces, not
  hidden screen-local product truth
- scene readiness language still follows the P014 Arrangement / Scene contract
  and does not imply automatic arranger behavior
- timing-trust copy still respects P012 fallback, manual-confirm,
  explicit-BPM, and locked-grid boundaries
- musician-facing proof language stays concise on perform surfaces and keeps
  detailed proof in inspect/docs surfaces
- Help and first-run surfaces remain readable without hiding the primary
  gesture route

## Explicit Deferrals

P015 does not claim:

- full product completion
- automatic scene-chain scheduling
- a second Scene Graph or hidden arrangement truth
- a product taste oracle
- arbitrary-source musical polish
- host-audio recording or long-session soak evidence
- full DAW/stem export readiness
- autonomous Ghost performance
- full source-derived MC-202 phrase planning
- final W-30 loop detection quality

Those remain later product or QA work and must reuse the existing Source Graph,
Session, Action Lexicon, queue/commit, observer, and audio-QA seams.

## Exit Decision Inputs

Use this checklist as input to the eventual P015 phase closeout. The closeout
still needs a phase-level review entry and a `docs/phase_definition_of_done.md`
status update once the final verification set passes.
