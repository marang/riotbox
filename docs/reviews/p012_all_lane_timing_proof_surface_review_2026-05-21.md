# P012 All-Lane Timing Proof Surface Review - 2026-05-21

- Review type: focused `review-codebase` pass for `RIOTBOX-871`
- Scope:
  - `Justfile` P012 / Recipe 15 proof targets
  - `docs/jam_recipes.md` Recipe 15 proof instructions
  - `docs/execution_roadmap.md` P012 deliverable note
  - `scripts/validate_auto_feral_grid_source_timing_pack.sh`
  - `scripts/correlate_generated_feral_grid_observer.sh`
  - observer/audio Source Timing summary and validator seams by contract search
- Current branch: `feature/riotbox-871-review-current-p012-all-lane-timing-proof-surface`

## Summary

The current P012 proof surface is coherent after `RIOTBOX-870`: the phase-level
gate now runs the existing observer/audio all-lane proof, the Recipe 2
observer/audio gate, and the Recipe 15 real-source Feral-grid auto/fallback
proof from one command.

No architecture fork or duplicate timing authority was found. The main remaining
risk is that the newly folded Recipe 15 portion still behaves like an optional
local recipe when fixture WAVs are missing. That is acceptable for ad-hoc local
exploration, but it is too soft for a phase-level proof command whose name says
the all-lane output path was proven.

## Findings

- **Location**: `scripts/validate_auto_feral_grid_source_timing_pack.sh:74`
- **Category**: scope
- **Severity**: major
- **Title**: Phase-level Recipe 15 proof can pass without real-source fixtures
- **Description**: The generalized Recipe 15 validator exits `0` when a required
  source WAV is missing. Through `scripts/validate_beat03_auto_feral_grid_pack.sh`
  and `Justfile:241`, that behavior now feeds `Justfile:463`
  `p012-all-lane-source-grid-output-proof`. This means the phase-level gate can
  report success without proving the Beat03, Beat08, Beat20, or DH_BeatC
  real-source auto/fallback contract on a checkout that lacks those files.
- **Suggestion**: Add a strict mode for phase-level use, for example
  `RIOTBOX_REQUIRE_RECIPE15_FIXTURES=1` or a `--require-source-fixtures` flag,
  and have `p012-all-lane-source-grid-output-proof` call Recipe 15 through that
  strict path. Keep the current skip behavior available for optional local
  recipe exploration if needed.

## Positive Checks

- **Location**: `Justfile:463`
- **Category**: scope
- **Severity**: note
- **Title**: P012 gate now composes the right proof families
- **Description**: The gate runs observer/audio Feral-grid correlation,
  Recipe 2 observer/audio validation, and Recipe 15 real-source auto/fallback
  validation. This is the right current shape for P012 because it combines
  control-path observer evidence, output-path lane metrics, and conservative
  source-timing trust boundaries.

- **Location**: `scripts/correlate_generated_feral_grid_observer.sh:99`
- **Category**: scope
- **Severity**: note
- **Title**: Observer/audio correlation checks both control and output evidence
- **Description**: The generated Feral-grid proof correlates observer events with
  rendered manifests under `--require-evidence`, including key outcomes, commit
  boundaries, Source Timing grid-use compatibility, downbeat-offset compatibility,
  anchor/groove alignment, and output-path issue emptiness.

- **Location**: `docs/jam_recipes.md:730`
- **Category**: scope
- **Severity**: note
- **Title**: Recipe documentation points users to the phase-level proof
- **Description**: Recipe 15 now explains that the P012 all-lane proof includes
  the same auto/fallback contract, which keeps the musician-facing recipe and
  phase-level QA command from drifting.

## Recommended Next Slice

Create a bounded follow-up to make the Recipe 15 portion strict when it is run
from the P012 phase-level gate. The functional impact for musicians and reviewers
is straightforward: `just p012-all-lane-source-grid-output-proof` should either
prove the real-source Beat03 / Beat08 / DH_BeatC / Beat20 auto/fallback contract
or fail loudly with a missing-fixture explanation. It should not silently pass
that part of the gate.

This is a better next slice than adding more examples because it hardens the
meaning of the command we now use as the phase-level all-lane proof.
