# P012 Source Timing QA Contract Review - 2026-05-20

## Scope

Review-codebase pass after the RIOTBOX-816 and RIOTBOX-817 Source Timing QA
slices.

Reviewed:

- `scripts/source_timing_example_probe_report.py`
- `scripts/validate_source_timing_probe_json.py`
- `scripts/fixtures/source_timing_example_probe_report/`
- `crates/riotbox-audio/tests/fixtures/source_timing_probe/`
- Source Timing recipes in `Justfile`
- Source Timing QA contract language in
  `docs/specs/source_timing_intelligence_spec.md`

This review did not change timing, analyzer, Session, action, JamAppState, or
audio-output behavior.

## Findings

### Major: Example report expectations can silently stop checking a score range

Location:

- `scripts/source_timing_example_probe_report.py:341`
- `scripts/source_timing_example_probe_report.py:349`
- `scripts/source_timing_example_probe_report.py:350`

Category: scope / correctness

`compare_number_range` treats `min` and `max` as optional and does not reject an
empty object or an inverted range. A malformed expectation such as
`"primary_beat_score": {}` would still report `ok` because no actual bound is
checked. A range such as `{"min": 0.8, "max": 0.2}` would always fail against
normal values, but the error would look like a detector regression rather than a
bad expectation fixture.

Impact:

The report itself is still reading the correct probe JSON fields, and
RIOTBOX-817 now clamps the producer-side JSON values. The remaining risk is in
the reviewer-facing expectation layer: a typo in the optional expectations file
can weaken or confuse the local regression surface while still producing a
successful report.

Suggestion:

Add strict expectation schema validation for numeric ranges: require at least
one of `min` or `max`, reject `min > max`, reject unknown range keys, and add one
or two invalid expectation fixtures to
`just source-timing-example-probe-report-fixtures`.

Follow-up:

- Create a bounded P012 follow-up for strict Source Timing example expectation
  range validation.

## Non-Findings

- The Source Timing probe JSON validator now rejects out-of-range top-level
  score and ratio evidence through a single helper, and generated source timing
  smokes already pass the same validator before their scenario-specific checks.
- The Python report and validator helpers remain below the soft review budget;
  no file split is useful right now.
- The `grid_use` recomputation in `validate_source_timing_probe_json.py` is an
  external compatibility check, not a runtime timing authority, and remains
  consistent with the spec boundary.
- The current Source Timing example report stays optional for local WAVs and
  reports missing files as skipped rows, preserving fresh-clone CI behavior.

## Recommended Follow-ups

1. Add strict schema validation for Source Timing example expectation range
   objects.
2. When the report grows again, consider moving table rendering and expectation
   validation into separate helpers before the script crosses the soft review
   budget.

## Verification Commands

```bash
nl -ba scripts/source_timing_example_probe_report.py | sed -n '1,430p'
nl -ba scripts/validate_source_timing_probe_json.py | sed -n '1,360p'
nl -ba Justfile | sed -n '130,325p'
nl -ba docs/specs/source_timing_intelligence_spec.md | sed -n '170,210p;548,572p'
```
