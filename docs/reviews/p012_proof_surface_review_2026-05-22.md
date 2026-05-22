# P012 Proof Surface Review - 2026-05-22

## Scope

Focused cadence review for `RIOTBOX-952` after `RIOTBOX-946` through
`RIOTBOX-950` expanded the compact P012 proof surfaces.

Reviewed current `main` after `RIOTBOX-950`:

- `docs/specs/source_timing_intelligence_spec.md`
- `scripts/write_p012_all_lane_proof_summary.py`
- `scripts/validate_p012_all_lane_proof_summary.py`
- `scripts/correlate_generated_feral_grid_observer.sh`

This review did not change analyzer, Session, action, JamAppState, UI,
observer, or audio-output behavior.

## Findings

### Minor - Generated-path phrase evidence is still hidden in the Markdown proof summary

- Location: `scripts/write_p012_all_lane_proof_summary.py:113`
- Location: `scripts/correlate_generated_feral_grid_observer.sh:136`
- Location: `scripts/correlate_generated_feral_grid_observer.sh:374`
- Location: `scripts/correlate_generated_feral_grid_observer.sh:489`
- Location: `docs/specs/source_timing_intelligence_spec.md:773`

The compact P012 Markdown summary now exposes generated-path cue/action,
grid-use compatibility, downbeat offset compatibility, downbeat ambiguity,
anchor alignment, and groove alignment. It still does not expose generated-path
`primary_phrase_count` or `primary_phrase_bar_count`.

The generated observer/audio gate already asserts those phrase fields for
cautious/manual-confirm, fallback, and locked-grid paths. The Source Timing spec
also says observer/audio summaries should preserve phrase count and phrase-bar
evidence beside `phrase_status`, so downstream QA can distinguish no phrase
grid, short-loop material, and stable preliminary phrase evidence without
opening the source manifest.

Impact: reviewers can now inspect most generated-path timing evidence from the
Markdown summary, but phrase evidence remains split: Recipe 15 rows show phrase
count/bars while generated Feral-grid observer/audio rows still require opening
JSON. That weakens the phase-level review artifact as the single compact
P012 readout.

Recommendation: add `Phrase count` and `Phrase bars` to the generated
Feral-grid observer/audio table in `scripts/write_p012_all_lane_proof_summary.py`
after `RIOTBOX-951` lands the equivalent TSV surface. Keep it display-only and
validator-backed; do not change analyzer behavior.

## Non-Findings

- No shadow timing authority was introduced. The Markdown and TSV surfaces read
  existing `output_path.source_timing` and alignment fields instead of
  recomputing policy locally.
- The strict generated Feral-grid gate still validates the JSON contract before
  copying summary artifacts.
- The P012 Markdown validator is intentionally snippet-based and brittle, but
  that brittleness is currently useful because it pins exact reviewer-facing
  rows for the phase proof.

## Recommended Next Slice

After `RIOTBOX-951` merges, add generated-path phrase count and phrase-bar
columns to the compact P012 Markdown summary and validator snippets.

Musician-facing effect:

- Reviewers can tell from one Markdown table whether a generated path is a
  short-loop/manual-confirm case, unavailable fallback, or locked phrase-grid
  case.
- The proof remains honest: phrase evidence is displayed as bounded QA evidence,
  not as a claim that Riotbox has production-grade arbitrary-audio phrase
  detection.

## Verification

Review-only slice. Verification:

```bash
git diff --check
```
