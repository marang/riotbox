# P012 Current Source Timing Spine Review - 2026-05-21

## Scope

Focused review for `RIOTBOX-861` to decide the next bounded P012 product slice
from the current implementation, not from stale review follow-ups.

Reviewed:

- `docs/execution_roadmap.md`
- `docs/phase_definition_of_done.md`
- `docs/specs/source_timing_intelligence_spec.md`
- `docs/specs/tui_screen_spec.md`
- `docs/reviews/p012_source_timing_qa_contract_review_2026-05-20.md`
- `docs/reviews/p012_source_timing_report_expectation_gate_review_2026-05-21.md`
- `docs/reviews/p012_source_timing_ui_observer_alignment_review_2026-05-21.md`
- `Justfile`
- `scripts/correlate_generated_feral_grid_observer.sh`
- `crates/riotbox-audio/src/bin/feral_grid_pack/bpm_decision_tests.rs`
- `crates/riotbox-app/src/bin/observer_audio_correlate/`

This review did not change analyzer, Session, action, JamAppState, or
audio-output behavior.

## Findings

No current blocker was found in the reviewed all-lane P012 Source Timing proof
spine.

The old review risks around source-timing path visibility, observer/audio
alignment, and generated Feral-grid proof should not be turned into duplicate
tickets without first checking current `main`. The current implementation
already proves the important policy paths and output surfaces:

- fallback timing remains visible as `static_default` /
  `source_timing_missing_bpm` with unavailable grid use and strict
  observer/audio evidence
- cautious short-loop timing remains visible as `source_timing` /
  `source_timing_needs_review_manual_confirm`, with compatible but not falsely
  locked observer/manifest grid-use and downbeat-offset evidence
- explicit BPM remains a user override and preserves source-timing BPM delta and
  agreement/disagreement evidence
- locked timing uses `source_timing_ready`, `locked_grid`, anchor/groove
  evidence, and lane output alignment

## Non-Findings

- The roadmap still points P012 at real source-timing confidence, shared
  Jam/Source/observer language, all-lane source-grid output proof, and honest
  fallback/manual/explicit-BPM paths.
- The generated Feral-grid observer/audio gate covers cautious/manual-confirm,
  user override, risky user override, fallback, locked-grid, and a strict
  mismatch failure path in one executable proof.
- `feral_grid_pack` has focused decision tests for ready source timing, weak
  fallback, manual-confirm rejection, cautious `NeedsReview` usage, ambiguous
  `NeedsReview` rejection, explicit BPM override, and missing/invalid source
  BPM labels.
- Observer/audio summary tests and fixtures assert source-grid drift, TR-909
  alignment, MC-202 alignment, W-30 alignment, W-30 loop closure, control-path
  presence, output-path presence, and strict evidence.
- The shared Jam Source Timing summary is now the right UI/observer language
  boundary. The TUI spec requires Jam, Help, Source, observer snapshots, and
  observer/audio QA to use that shared compact contract instead of inventing
  screen-local timing language.

## Recommended Next Slice

Move from QA-contract reinforcement to real-source timing confidence:

1. Pick one real local example row that is currently below the desired readiness
   boundary but musically plausible for P012.
2. Tighten the Rust timing evidence or readiness classification enough to
   improve that row without claiming production-grade arbitrary-audio detection.
3. Prove the result through the existing Source Timing example report, Jam/Source
   timing cue or observer snapshot when applicable, and the relevant
   observer/audio output gate.
4. Keep the explicit fallback and manual-confirm behavior intact for weak or
   ambiguous sources.

Musician-facing effect:

- A real loop should more often show a useful, honest timing cue instead of
  looking like generic fallback.
- When Riotbox is confident, generated TR-909, MC-202, and W-30 support should
  feel locked to the source grid with fewer manual BPM escapes.
- When Riotbox is not confident, the musician should still see why the grid was
  not trusted and should not hear a falsely locked rebuild.

## Evidence References

- Roadmap near-next P012 direction: `docs/execution_roadmap.md:571`
- Phase 2 timing-ready analysis bar: `docs/phase_definition_of_done.md:68`
- Source Timing generated Feral-grid gate contract:
  `docs/specs/source_timing_intelligence_spec.md:780`
- TUI shared timing-language contract: `docs/specs/tui_screen_spec.md:83`
- All-lane gate entrypoint: `Justfile:445`
- Generated Feral-grid cautious/manual-confirm path:
  `scripts/correlate_generated_feral_grid_observer.sh:50`
- Generated Feral-grid user-override path:
  `scripts/correlate_generated_feral_grid_observer.sh:150`
- Generated Feral-grid fallback path:
  `scripts/correlate_generated_feral_grid_observer.sh:268`
- Generated Feral-grid locked-grid path:
  `scripts/correlate_generated_feral_grid_observer.sh:362`
- Generated Feral-grid strict mismatch failure path:
  `scripts/correlate_generated_feral_grid_observer.sh:466`
- Feral-grid BPM decision tests:
  `crates/riotbox-audio/src/bin/feral_grid_pack/bpm_decision_tests.rs:5`
- Observer/audio lane alignment and loop-closure fixture evidence:
  `crates/riotbox-app/src/bin/observer_audio_correlate/tests.rs:63`
- MC-202 source-grid alignment summary evidence:
  `crates/riotbox-app/src/bin/observer_audio_correlate/summary_smoke_tests.rs:80`

## Verification Commands

```bash
nl -ba scripts/correlate_generated_feral_grid_observer.sh | sed -n '1,470p'
nl -ba crates/riotbox-audio/src/bin/feral_grid_pack/bpm_decision_tests.rs | sed -n '1,180p'
nl -ba crates/riotbox-app/src/bin/observer_audio_correlate/tests.rs | sed -n '55,180p'
nl -ba crates/riotbox-app/src/bin/observer_audio_correlate/summary_smoke_tests.rs | sed -n '80,125p'
nl -ba Justfile | sed -n '240,310p;320,348p;438,448p'
nl -ba docs/execution_roadmap.md | sed -n '560,585p'
nl -ba docs/phase_definition_of_done.md | sed -n '65,78p'
```
