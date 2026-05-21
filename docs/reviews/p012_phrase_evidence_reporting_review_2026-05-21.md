# P012 Phrase Evidence Reporting Review - 2026-05-21

Scope: broad review after `RIOTBOX-897` through `RIOTBOX-901`, focused on the
Source Timing warning / phrase-evidence reporting path across Core readiness,
probe JSON/text, local example reports, generated Feral-grid manifests, compact
P012 proof summaries, observer/audio summaries, validators, and the Source Timing
spec.

## Summary

The main Source Timing phrase-evidence path is coherent:

- Core readiness derives `primary_phrase_count` and `primary_phrase_bar_count`
  from the selected timing hypothesis, not from app-local state.
- Probe JSON/text, example reports, generated Feral-grid manifests/text reports,
  and the compact P012 all-lane proof summary now expose those counts.
- The generated Feral-grid manifest and P012 proof-summary validators now fail
  if the phrase-count contract disappears.
- The new reporting fields do not change readiness, grid-use, lane, Session,
  replay, JamAppState, or audio-output behavior.

Two bounded follow-ups remain before the reporting path is fully aligned.

## Findings

### 1. Observer/audio summaries drop manifest phrase-count evidence

- Location: `crates/riotbox-app/src/bin/observer_audio_correlate/summary_build.rs:45`
- Location: `crates/riotbox-app/src/bin/observer_audio_correlate/summary_build.rs:288`
- Location: `crates/riotbox-app/src/bin/observer_audio_correlate/summary_render.rs:140`
- Category: scope
- Severity: major
- Title: Observer/audio `output_path.source_timing` still serializes only phrase status
- Description: Generated Feral-grid manifests now preserve
  `primary_phrase_count` and `primary_phrase_bar_count`, and the compact P012
  proof summary reads those fields directly from the manifests. The
  observer/audio correlation summary still collects `phrase_status` but not the
  two phrase-count fields, then serializes `output_path.source_timing` without
  them. That leaves one downstream QA surface unable to distinguish no phrase
  grid, short-loop material, and stable preliminary phrase evidence without
  reopening the manifest.
- Suggestion: Add `primary_phrase_count` and `primary_phrase_bar_count` to
  `ManifestSourceTimingReadiness` in the observer/audio correlate path, serialize
  them in `output_path.source_timing`, validate them in
  `scripts/validate_observer_audio_summary_json.py`, and update the relevant
  observer/audio fixtures.

### 2. Local example-report expectations cannot pin phrase counts

- Location: `scripts/source_timing_example_probe_report.py:199`
- Location: `scripts/source_timing_example_expectations.py:10`
- Location: `scripts/source_timing_example_expectations.py:65`
- Category: scope
- Severity: minor
- Title: Example report renders phrase counts but expectation schema cannot assert them
- Description: The local Source Timing example report table now renders `Phrase
  count` and `Phrase bars`, but `source_timing_example_expectations.py` does not
  include `primary_phrase_count` or `primary_phrase_bar_count` in
  `EXPECTATION_KEYS` and does not compare them in `expectation_issues`. The P012
  all-lane proof validator catches the current generated Recipe 15 values, but
  the optional local example-report regression path can still drift in phrase
  counts while reporting `ok`.
- Suggestion: Add exact integer expectation support for
  `primary_phrase_count` and `primary_phrase_bar_count`, update the local
  Beat/DH expectations where the values are stable enough, and keep the existing
  missing-local-WAV behavior unchanged.

## Checked Areas

- Core readiness and confidence: `source_timing_candidate_confidence_report(...)`
  computes phrase counts from the selected primary hypothesis, and
  `source_timing_probe_readiness_report(...)` carries them through the readiness
  contract.
- Probe CLI: `source_timing_probe` JSON/text includes the phrase counts and the
  probe JSON validator requires non-negative integers.
- Generated Feral-grid manifests: manifest serialization, text reports, and
  listening-manifest validation preserve the same fields.
- Compact P012 proof summary: Recipe 15 rows now show phrase count / bar count,
  and the summary validator requires the current generated values.
- Spec: `docs/specs/source_timing_intelligence_spec.md` now records the generated
  Feral-grid phrase-evidence manifest contract.

## Follow-ups

- Add phrase-count evidence to observer/audio `output_path.source_timing`
  summaries.
- Add phrase-count expectation checks to the local Source Timing example report
  expectation schema.
