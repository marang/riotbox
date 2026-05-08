# Source Timing BPM Candidates

Date: 2026-05-07
Project: `P012 | Source Timing Intelligence`
Status: initial probe BPM-candidate, beat-period, downbeat-accent, and drift scoring spike

## Purpose

`crates/riotbox-core/src/source_graph/timing_probe_candidates.rs` turns
deterministic probe onset times into preliminary BPM candidate hypotheses.

The candidate path reports:

- primary BPM estimate from scored beat-period candidates
- primary beat/bar candidate grids
- half-time and double-time alternatives when they fit the policy range
- downbeat-accent phase scoring from probe-window onset strength
- 4/8/16/32-bar source-grid drift reports when enough source material exists
- preliminary 4-bar phrase candidates when bar timing is stable and long enough
- beat-period evidence summaries for QA explanation of candidate selection
- downbeat-phase evidence summaries for QA explanation of bar-phase selection
- combined timing readiness summaries for QA / future-TUI review of whether the
  current probe result is unavailable, weak, needs-review, or ready
- ambiguous downbeat and phrase-uncertain warnings when evidence stays weak
- fixture-like PCM WAV probe evidence that reaches the same candidate path

## Gate

Run:

```bash
just source-timing-bpm-candidates
```

The gate uses synthetic onset spacing plus generated PCM WAV probe fixture
paths to prove that a 120 BPM impulse train and a fixture-like pulse source can
produce a BPM candidate, accent evidence can select a clearer downbeat phase,
stable-grid drift can be measured across short and longer windows, and sparse
or ambiguous onsets degrade instead of claiming timing lock. A focused
phrase-grid gate proves that stable bar evidence can produce preliminary
4-bar phrase spans while short material stays phrase-uncertain.

```bash
just source-timing-phrase-grid
```

A focused beat-evidence gate proves that the dominant onset interval evidence
is reportable as candidate count, primary BPM/period, primary score,
matched-onset ratio, median-distance ratio, alternate-candidate count, and a
stable / weak / ambiguous / unavailable status. It also checks the generated
WAV bridge for stable long accented 120 BPM material.

```bash
just source-timing-beat-evidence
```

A focused downbeat-evidence gate proves that primary phase offset, primary
score, alternate-phase count, and stable / weak / ambiguous / unavailable
status are reportable without consumers inspecting raw phase scoring internals.

```bash
just source-timing-downbeat-evidence
```

A focused readiness-report gate proves that candidate confidence, beat-period
evidence, and downbeat-phase evidence can be summarized without becoming a
separate timing authority.

```bash
just source-timing-readiness-report
```

That gate also includes generated real-loop-like 128 BPM drum fixtures:

- a flat fixture that deliberately keeps downbeat and phrase evidence weak while
  preserving a stable BPM estimate and half-time ambiguity
- an accented fixture that proves clear beat, downbeat, and phrase evidence can
  classify as `ready` through the same PCM WAV probe path when evaluated with a
  bounded 80-180 BPM dance-loop auto-readiness policy

Together they give later detector work both a negative and positive guardrail, so
real example readiness can improve without loosening policy blindly or committing
external WAV files.

The timing candidate policy has two named profiles:

- `broad_research`: the default diagnostic policy, intentionally wide at
  55-240 BPM so half-time, double-time, and near-period ambiguity remain visible
  during detector research
- `dance_loop_auto_readiness`: a narrower 80-180 BPM policy with a stricter
  period-ambiguity margin for Feral grid auto-BPM trust, so a clearly locked
  dance loop can become `ready` without changing the broader research policy

Generated Feral grid listening packs should carry this readiness summary and
the named readiness policy profile in their manifest so example audio can be
judged against timing evidence instead of only against rendered level metrics.

## Boundary

This is not a production BPM/downbeat detector. It is an intentionally bounded
candidate-shape proof so later real-source detection can reuse the same
`TimingModel` contract and ambiguity behavior.
