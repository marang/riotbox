# Source Timing Candidate Confidence Report

Date: 2026-05-07
Project: `P012 | Source Timing Intelligence`
Status: initial probe timing confidence report seam

## Purpose

`source_timing_candidate_confidence_report(...)` turns the current probe-derived
`TimingModel` into compact QA evidence.

The report captures:

- primary BPM and BPM confidence
- effective timing quality and degraded policy
- total hypotheses plus alternate downbeat, half-time, and double-time counts
- primary downbeat confidence
- primary source-grid drift status, available window count, drift metrics, and
  drift confidence
- primary phrase-grid status, phrase count, covered bar count, and phrase
  confidence
- warning codes and manual-confirm requirement

## Gate

Run:

```bash
just source-timing-candidate-confidence-report
```

The gate proves that ambiguous BPM candidates expose alternate downbeat and
tempo ambiguity, while degraded sparse probes report no primary BPM and stay in
manual-review territory. It also proves that stable 4/8-bar drift and high
source-grid drift are summarized without requiring consumers to inspect raw
timing hypotheses. Phrase-grid assertions distinguish stable preliminary
4-bar phrase spans from too-short, ambiguous-downbeat, and high-drift material.

## Boundary

This is not a user-facing confidence UI and not a production detector. It is a
reviewable regression seam for later source timing detector improvements.
