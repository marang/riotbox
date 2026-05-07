# Source Timing Downbeat Ambiguity

Date: 2026-05-07
Project: `P012 | Source Timing Intelligence`
Status: initial probe BPM downbeat phase scaffold

## Purpose

`crates/riotbox-core/src/source_graph/timing_probe_candidates.rs` now scores
simple beat-phase candidates on top of probe-derived BPM hypotheses.

The scaffold reports:

- a primary bar grid using the strongest preliminary downbeat phase
- `AlternateDownbeat` hypotheses when multiple phases are similarly plausible
- explicit `AmbiguousDownbeat` warnings while scoring is only onset-presence
  based

## Gate

Run:

```bash
just source-timing-downbeat-ambiguity
```

The gate uses synthetic onset timing to prove that equal beat-phase evidence
stays ambiguous, while a clearer repeated phase can keep the primary bar grid
anchored without emitting alternate downbeat hypotheses.

## Boundary

This is not a production downbeat detector. It is a contract proof that later
source-timing analysis can preserve phase ambiguity instead of silently choosing
one bar-start interpretation.
