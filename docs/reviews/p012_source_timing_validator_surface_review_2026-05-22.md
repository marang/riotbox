# P012 Source Timing Validator Surface Review - 2026-05-22

## Scope

Focused broad-review cadence pass after RIOTBOX-956 through RIOTBOX-960 tightened
Source Timing grid-count reporting and validator contracts.

Reviewed current `main` after RIOTBOX-960:

- `scripts/validate_source_timing_probe_json.py`
- `scripts/validate_observer_audio_summary_json.py`
- `scripts/validate_source_timing_grid_use_contract_fixtures.py`
- `scripts/source_timing_example_probe_report.py`
- `docs/specs/source_timing_intelligence_spec.md`

This review did not change analyzer, Session, action, JamAppState, UI,
observer schema, or audio-output behavior.

## Findings

### Minor - Observer/audio validator has become the next Source Timing review-cost hotspot

- Location: `scripts/validate_observer_audio_summary_json.py:706`
- Location: `scripts/validate_observer_audio_summary_json.py:921`
- Category: scope
- Severity: minor

The observer/audio summary validator now owns general summary schema checks,
observer-side Source Timing checks, manifest-side Source Timing checks, alignment
checks, phrase evidence checks, and count/status consistency checks in one file.
The latest count validator is correct and covered, but the file is now over
1,200 lines and Source Timing-specific validator logic is mixed with unrelated
summary and lane-recipe validation.

Impact: future P012 validator slices will continue to need large-file context
for small Source Timing contract changes, increasing review and agent context
cost.

Suggestion: extract observer-side and manifest-side Source Timing validation
helpers into a narrow module or companion script only when the next validator
change touches this area again. Keep behavior unchanged and move tests/fixtures
with the helper.

### Minor - Grid-use contract fixture counts are implicit rather than owned by cases

- Location: `scripts/validate_source_timing_grid_use_contract_fixtures.py:249`
- Location: `scripts/validate_source_timing_grid_use_contract_fixtures.py:258`
- Category: scope
- Severity: minor

`GridUseCase` owns status, readiness, policy, BPM, and phrase state, but the new
probe-only beat/bar counts are inferred inside `apply_timing_fields` from
`beat_status`, `downbeat_status`, and `phrase_status`. This is fine for the
current generated fixture cases, but it makes future count-sensitive cases less
explicit than the rest of the contract table.

Impact: future grid-use fixture work could accidentally encode count assumptions
in helper branching instead of in the case data that reviewers read first.

Suggestion: when another grid-use contract case is added or count behavior
changes, move primary beat/bar counts into `GridUseCase` fields and set them per
case, matching the explicit style used for policy and status fields.

## Non-Findings

- The latest Source Timing probe JSON validator rejects contradictory beat/bar
  count evidence and keeps grid-use derivation centralized in the existing
  validator path.
- The local example probe report reads probe JSON fields directly instead of
  recomputing count/status policy in the renderer.
- The observer/audio summary validator now matches the user-session observer
  count/status contract for the locked observer path.
- No new shadow timing authority was introduced.

## Recommended Next Slice

Create one bounded cleanup ticket for the next Source Timing validator pass:
make grid-use contract fixture beat/bar counts explicit in `GridUseCase`.

Musician-facing effect:

- None directly on sound or UI.
- The proof harness becomes easier to trust because future grid-use/count
  variants are visible in the fixture table instead of hidden in helper logic.

## Verification

Review-only slice. Verification:

```bash
git diff --check
```
