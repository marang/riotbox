# RIOTBOX-1426 Tempo-Guided Source Phase Review

Date: 2026-07-27
Phase: P023 / Controlled Expansion
Classification: `contract_enabler`
Directly enabled audible follow-up: RIOTBOX-1422 H24 fresh exact-path attempt

## Outcome

The branch adds a typed `TempoGuided` Source Graph timing hypothesis for the
bounded case where independently supplied tempo disagrees with or cannot use
the Rust analyzer primary. Tempo remains external; downbeat phase is selected
only from actual source onset phases. The selector requires onset/grid support,
complete-bar recurrence, a winning phase margin, and bounded drift.

The selected grid persists through the existing Source Graph reference and
Session restore. It is observer-labeled `source_tempo_guided`, uses
`locked_grid`, and does not fabricate a musician
`source_timing.confirm_grid` action. Matching-primary BPM and explicit
BPM-plus-downbeat `Manual` routes remain unchanged.

No W-30 DSP, hit-shaper selector, `0.18` / `1.40` / `0.30` ownership gate, or
realtime callback changed. No review WAV was generated and no human verdict was
requested.

## Development Evidence

The local matrix covers seven consumed/development CC0 cases across five source
families:

| Source | Family | Timing result |
| --- | --- | --- |
| 8-Bit Victory Loop | tonal riff | `TempoGuided` selected at project-derived 180 BPM |
| Drama | dense break | rejected: insufficient complete-bar phase support |
| Get Equipped | sparse drums | rejected: excessive drift for unverified 120 BPM challenge |
| Rain on Window | pad/noise | rejected: insufficient onsets |
| Metal thud2 | bad timing | rejected: insufficient material |
| can be so beautiful | dense break | existing matching-primary route preserved |
| Horde War Drums | dense break | existing matching-primary route preserved |

Victory selects a `0.010667 s` source-backed phase with `4/11` complete-bar
hits, `3.11 ms` mean drift, and a `0.10493` winning score margin. Drama retains
its mismatch failure with only `2/7` complete-bar hits and no winning margin.
Machine-readable evidence is frozen in
`docs/benchmarks/w30_tempo_guided_development_v1.json`.

The restored H24 sources — Lucid Trigger and NES Chopin — plus untouched Cave,
Sector, and bad-timing reserve were not accessed.

## Branch Review

The general and Rust-specific branch review found four actionable issues:

1. The initial phase-score margin equaled the constructive accent difference
   and could reject a clear positive through float rounding. The margin is now
   an explicit documented `0.020`, while flat accents still reject.
2. Initial bar grids could include a truncated final bar. Selection and output
   now count and emit complete bars only.
3. Product callers could directly select a constructed `TempoGuided`
   hypothesis. Analysis plus installation now use one atomic Core API; rejected
   evidence leaves the graph unchanged.
4. The new implementation plus tests crossed the Rust module-size review
   signal. Tests now live in the real child module
   `timing_tempo_guided::tests`, not a textual `include!` shard.

After those fixes, no unresolved correctness, architecture, realtime-safety,
test, or documentation finding remains. Analysis is ingest/preflight
control-plane work and adds no callback allocation, I/O, locking, or model call.

No `ActionCommand` was added. Queue/commit and replay behavior are intentionally
not invented for machine analysis: the selected hypothesis is persisted in the
existing Source Graph, the Session retains that graph identity, restore uses
the same hypothesis, and no user-confirmation record is forged.

## Verification

- synthetic non-zero phase, identity, sparse-tonal, flat-accent,
  insufficient-onset, complete-bar, deterministic, and preservation tests:
  pass
- Source Graph save/restore and no-fake-confirmation regression: pass
- observer origin regression: pass
- W-30 preflight route/product-phase regressions: pass
- seven-case / five-family local development matrix: pass
- final `just ci`: pass

## Remaining Gate

RIOTBOX-1426 proves trusted timing reachability, not exact W-30 hit-shaper
selection or musical quality. After this branch lands, RIOTBOX-1422 may freeze
the mechanism and access the restored reserve exactly once as H24. Candidate
audio and structured listening remain disallowed until the fresh exact-path
timing, Hard recipe, callback calibration, and technical gates all pass.
