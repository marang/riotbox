# P012 Source Timing UI / Observer Alignment Review - 2026-05-21

## Scope

Focused broad-review cadence pass after RIOTBOX-829 through RIOTBOX-838 tightened
Source Timing expectation gates, observer/audio alignment, generated Feral-grid
proof, and musician-facing downbeat-offset visibility.

Reviewed:

- `crates/riotbox-core/src/view/jam/source_timing_summary.rs`
- `crates/riotbox-core/src/view/jam/source_timing_summary_tests.rs`
- `crates/riotbox-app/src/ui/source_timing_panel.rs`
- `crates/riotbox-app/src/ui/source_trust_summary.rs`
- `crates/riotbox-app/src/observer.rs`
- `crates/riotbox-app/src/bin/observer_audio_correlate/`
- `scripts/validate_observer_audio_summary_json.py`
- Source Timing UI / observer contract text in
  `docs/specs/source_timing_intelligence_spec.md`

This review did not change analyzer, Session, action, JamAppState, or
audio-output behavior.

## Findings

### Major: Validator does not check anchor/groove alignment status against issues

Location:

- `scripts/validate_observer_audio_summary_json.py:415`
- `scripts/validate_observer_audio_summary_json.py:429`
- `scripts/validate_observer_audio_summary_json.py:696`
- `scripts/validate_observer_audio_summary_json.py:719`

Category: scope / QA contract

`source_timing_alignment` has validator-side consistency checks for
`grid_use_compatibility` and `downbeat_offset_compatibility`, including whether
the expected mismatch issue is present. The sibling
`source_timing_anchor_alignment` and `source_timing_groove_alignment` validators
currently only check shape: `status`, optional observer/manifest evidence, and a
string `issues` list.

Impact:

The Rust correlator derives `mismatch` from non-empty issues today, so generated
summaries are internally coherent. The standalone JSON validator, however, would
accept malformed external or edited summaries such as `status: aligned` with a
non-empty anchor issue list, or `status: mismatch` with no issue. That weakens the
observer/audio summary JSON contract exactly where P012 now uses anchor and
groove alignment as strict evidence.

Suggestion:

Add validator helpers that require:

- `status == "mismatch"` iff `issues` is non-empty for anchor/groove alignment.
- non-mismatch statuses must not carry anchor/groove issue strings.
- mismatch statuses must carry at least one issue string with the corresponding
  `source_timing_anchor_alignment.` or `source_timing_groove_alignment.` prefix.

Add tiny valid/invalid fixture coverage or focused unit-style validator cases.

## Non-Findings

- `SourceTimingSummaryView` is now the shared presenter for cue, quality,
  degraded policy, grid use, primary warning, primary downbeat offset, anchor
  counts, and groove preview. The user-session observer reads those shared values
  instead of recalculating offset/anchor/groove summary state locally.
- Raw beat/downbeat/phrase statuses in TUI Source and observer snapshots remain
  diagnostic fields derived from Source Graph timing state, which matches the
  current spec boundary for detailed diagnostics.
- The Source Timing summary file split resolved the immediate Rust review-budget
  concern: production summary, summary tests, observer/audio alignment, and split
  alignment tests are all below the 500-line soft budget.
- No new `ActionCommand`, Session/replay state, `JamAppState` state, or audio
  behavior was introduced by the reviewed P012 offset/UI slices.

## Recommended Follow-ups

1. Add anchor/groove alignment status-vs-issue validation to
   `scripts/validate_observer_audio_summary_json.py`.

## Verification Commands

```bash
nl -ba crates/riotbox-core/src/view/jam/source_timing_summary.rs | sed -n '1,320p'
nl -ba crates/riotbox-app/src/ui/source_timing_panel.rs | sed -n '1,150p'
nl -ba crates/riotbox-app/src/observer.rs | sed -n '160,245p'
nl -ba crates/riotbox-app/src/bin/observer_audio_correlate/source_timing_alignment.rs | sed -n '1,340p'
nl -ba scripts/validate_observer_audio_summary_json.py | sed -n '400,455p;535,570p;690,760p'
wc -l crates/riotbox-core/src/view/jam/source_timing_summary.rs crates/riotbox-core/src/view/jam/source_timing_summary_tests.rs crates/riotbox-app/src/bin/observer_audio_correlate/source_timing_alignment.rs crates/riotbox-app/src/bin/observer_audio_correlate/source_timing_alignment_*.rs crates/riotbox-app/src/observer.rs crates/riotbox-app/src/ui/source_timing_panel.rs
```
