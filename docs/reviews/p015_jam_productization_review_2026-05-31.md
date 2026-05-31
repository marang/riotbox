# P015 Jam Productization Review - 2026-05-31

Scope:

- `crates/riotbox-app/src/ui/jam_perform_layout.rs`
- `crates/riotbox-app/src/ui/footer_help_perform_lines.rs`
- `crates/riotbox-app/src/ui/source_trust_summary/arrangement.rs`
- `docs/jam_recipes.md`
- `docs/screenshots/jam_first_30_seconds_baseline.txt`
- `docs/screenshots/jam_taste_proof_glossary.md`

Context:

- Broad review cadence after the RIOTBOX-1037, RIOTBOX-1050, RIOTBOX-1051,
  RIOTBOX-1052, and RIOTBOX-1053 P015 slices.
- Focused on whether Jam productization language respects the P012 Source
  Timing and P014 Arrangement / Scene boundaries.

## Findings

### RIOTBOX-1054 - Recipe 16 still promoted scene jump too broadly

- Location: `docs/jam_recipes.md`, Recipe 16
- Category: scope
- Severity: major
- Title: Recipe text bypassed the new first-run timing-trust boundary
- Description: RIOTBOX-1053 made the first-result Start Here next-move cue avoid
  promoting `[y]` scene jump while Arrangement / Scene readiness is cautious or
  fallback-backed, but Recipe 16 still told the musician to press `y` whenever a
  scene jump was offered.
- Suggestion: Update Recipe 16 so `y` is the first choice only when Jam says
  `taste scene-ready`; cautious, sketch, or unknown taste should steer the user
  toward `[g] follow` or `[f] fill` first.
- Closure: fixed in RIOTBOX-1054.

## No Finding

- The current taste/proof text is still derived from the shared Jam view model
  and Session-backed Arrangement / Scene contract, not screen-local inference.
- No new ActionCommand, Session truth, replay truth, or audio path was introduced
  by the reviewed P015 productization slices.
- The relevant Rust files remain under the 500-line soft review budget.
