# P012 Source Timing Readiness / Warning Broad Review - 2026-05-21

## Scope

Broad-review cadence pass for `RIOTBOX-896` after RIOTBOX-892 through
RIOTBOX-895 tightened Source Timing readiness actionability labels, stable
locked-grid readiness, and sparse-onset warning language.

Reviewed:

- `crates/riotbox-core/src/source_graph/timing_probe_candidates/`
- `crates/riotbox-core/src/source_graph/timing_probe_diagnostics.rs`
- `crates/riotbox-core/src/view/jam/source_timing_summary.rs`
- `crates/riotbox-app/src/source_timing_cues.rs`
- `crates/riotbox-app/src/observer.rs`
- `crates/riotbox-app/src/ui/source_trust_summary.rs`
- `crates/riotbox-audio/src/bin/source_timing_probe.rs`
- Source Timing probe, observer/audio, user-session observer, fixture-report,
  example-report, and P012 proof validators under `scripts/`
- `docs/specs/source_timing_intelligence_spec.md`

This review did not change analyzer, Session, action, JamAppState, or
audio-output behavior.

## Findings

### Major: Jam Trust warning line bypasses shared primary-warning priority

- **Location**: `crates/riotbox-app/src/ui/source_trust_summary.rs:163`
- **Category**: scope / UI contract
- **Severity**: major
- **Title**: Jam Trust source-timing warning can disagree with the shared summary

`SourceTimingSummaryView` now chooses the single musician-facing
`primary_warning` through the documented priority order, so `drift_high`,
`ambiguous_downbeat`, and `sparse_onsets` are surfaced before less actionable
warnings. The observer snapshot uses that shared summary value through
`timing.primary_warning`, but the Jam Trust warning line still reads
`graph.timing.warnings.first()` and maps that raw first warning locally.

Impact:

The same source can show different "primary" timing warnings across Jam Trust,
Source Timing panel, observer NDJSON, and observer/audio QA if the raw warning
storage order does not match the documented priority. This is especially visible
after `sparse_onsets`, because sparse pads currently carry multiple warnings and
the most useful musician-facing explanation should be selected by priority, not
insertion order.

Suggestion:

Make `source_trust_summary.rs` read the shared summary's `primary_warning` or a
small shared accessor instead of inspecting `graph.timing.warnings.first()`. Add
a UI test with warnings ordered `low_timing_confidence`, `weak_kick_anchor`,
`sparse_onsets` and assert the Jam Trust line renders
`timing warning sparse_onsets`.

### Major: Fixture-report JSON validator does not accept SparseOnsets issues

- **Location**: `scripts/validate_source_timing_fixture_report_json.py:33`
- **Category**: scope / QA contract
- **Severity**: major
- **Title**: New typed warning is missing from the fixture-report issue whitelist

`TimingWarningCode::SparseOnsets` is now part of the core warning enum and is
emitted by the diagnostic and BPM-candidate fallback paths. The Source Timing
fixture-report validator still whitelists object-style
`{ "missing_warning": ... }` values without `SparseOnsets`, so a valid
fixture-evaluation report that requires the new warning would be rejected.

Impact:

The current probe/example validators already pass sparse-onset examples, but the
older fixture-evaluation report contract is one warning behind the Rust model.
That weakens P012's ability to turn sparse-source behavior into fixture
expectations without touching validator code again.

Suggestion:

Add `SparseOnsets` to `TIMING_WARNING_CODES` and extend the fixture-report
validator fixtures with a valid object issue that uses
`{ "missing_warning": "SparseOnsets" }`.

## Non-Findings

- The readiness boundary is correctly stricter than the broad degraded-policy
  label. `source_timing_probe_readiness_report(...)` promotes cautious timing to
  `Ready` only when the combined report has primary BPM, stable beat/downbeat,
  stable drift, stable phrase, no warnings, and no alternate evidence.
- Runtime Rust producers for probe CLI and generated Feral-grid manifests now
  derive readiness cue/actionability through the core
  `source_timing_readiness_labels(...)` / report helper path.
- `source_timing_grid_use(...)` remains the shared Rust policy source for
  readiness-report consumers, while Python validators independently recompute the
  same contract as external compatibility checks.
- Observer snapshots preserve both the prioritized `primary_warning_code` and
  the full raw `warning_codes` list, matching the spec's focus-plus-diagnostics
  contract.
- The reviewed Rust files are below the repo's roughly 500-line soft review
  budget; no semantic split is currently forced by file size alone.

## Recommended Follow-ups

1. Align the Jam Trust source-timing warning line with
   `SourceTimingSummaryView.primary_warning`.
2. Add `SparseOnsets` support and fixture coverage to the source-timing
   fixture-report JSON validator.
3. Keep the next implementation tickets small; both findings are bounded
   contract-alignment slices and do not require audio-output changes.

## Verification Commands

```bash
nl -ba crates/riotbox-core/src/source_graph/timing_probe_candidates/readiness_report.rs | sed -n '1,110p'
nl -ba crates/riotbox-core/src/source_graph/timing_probe_candidates/grid_use_policy.rs | sed -n '1,170p'
nl -ba crates/riotbox-core/src/source_graph/timing_probe_diagnostics.rs | sed -n '1,130p'
nl -ba crates/riotbox-core/src/view/jam/source_timing_summary.rs | sed -n '1,340p'
nl -ba crates/riotbox-app/src/observer.rs | sed -n '170,255p'
nl -ba crates/riotbox-app/src/ui/source_trust_summary.rs | sed -n '130,190p;330,372p'
nl -ba crates/riotbox-audio/src/bin/source_timing_probe.rs | sed -n '160,210p;286,352p'
nl -ba scripts/validate_source_timing_probe_json.py | sed -n '1,180p;260,340p'
nl -ba scripts/validate_source_timing_fixture_report_json.py | sed -n '1,85p;140,175p'
nl -ba scripts/validate_observer_audio_summary_json.py | sed -n '1,90p;300,338p;1020,1078p'
nl -ba scripts/validate_user_session_observer_ndjson.py | sed -n '130,200p;380,430p'
wc -l crates/riotbox-core/src/source_graph/timing_probe_candidates/*.rs crates/riotbox-core/src/source_graph/timing_probe_diagnostics.rs crates/riotbox-core/src/view/jam/source_timing_summary.rs crates/riotbox-app/src/source_timing_cues.rs crates/riotbox-app/src/observer.rs crates/riotbox-app/src/ui/source_trust_summary.rs crates/riotbox-audio/src/bin/source_timing_probe.rs scripts/validate_source_timing_probe_json.py scripts/source_timing_example_probe_report.py scripts/validate_p012_all_lane_proof_summary.py
```
