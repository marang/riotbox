# RIOTBOX-1425 W-30 Hit-Shaper Reachability Review

Date: 2026-07-27
Phase: P023 / Controlled Expansion
Classification: `contract_enabler`
Directly enabled audible follow-up: RIOTBOX-1422 H22 exact-path holdout and
structured listening

## Outcome

The branch replaces aggregate low-impact ownership with deterministic
source-local candidate selection. Each active source onset receives a bounded
adaptive attack and following-body window. The selector ranks the candidate's
weakest normalized ownership margin while preserving the existing `0.18`,
`1.40`, and `0.30` floors.

The selected role is typed as `transient_low_body`, explicitly not bass. The
decision, candidate count, selected slot/onset, window lengths, and three ratios
are present in app state, the coherent realtime snapshot, observer output, and
`riotbox.w30_reachability_preflight.v1`.

No review WAV was generated, no human verdict was requested, and the frozen H22
reserve was not accessed.

## Development Evidence

The local ignored matrix covers Beat03 plus all seven registered development
sources across seven registered families:

| Source | Family | Result | Attack/body |
| --- | --- | --- | ---: |
| Beat03 | dense break Golden Path | existing exact stable | `2.603159` |
| Bertsz | dense full mix | existing exact stable | `1.949662` |
| Cinameng | dense break | unavailable; gate preserved | `1.353822` |
| Marwan | sparse drums | newly exact | `1.411305` |
| Fupi | tonal riff | newly exact | `1.697597` |
| Isaiah658 | pad/noise | texture policy; no low-impact claim | n/a |
| KillerFishRed | weak source | unavailable | n/a |
| laleksic tap water | bad timing/source suitability | unavailable | n/a |

Repeat projections are identical. The matrix produces seven distinct selection
signatures, so the new path does not collapse these sources into one fixed
decision.

The exact Beat03 product preflight also passes through Source Graph, Session,
queue/commit, capture promotion, internal resample, Hard projection, and exact
callback calibration. It reports `source_hit_shaper_v3`,
`transient_low_body`, six evaluated candidates, and no blockers.

Machine-readable evidence is frozen in
`docs/benchmarks/w30_hit_shaper_reachability_development_v1.json`.

## Branch Review

The general and Rust-specific branch review found two actionable issues:

1. A recipe-only state could have claimed exact applicability while its new
   typed role and decision disagreed. The shared applicability predicate now
   requires the complete consistent tuple, with regression coverage.
2. Adding the selector and matrix tests to the existing large textual include
   would have increased review cost. The new behavior now lives in the real
   semantic `projection::w30_low_impact` module; the legacy include is smaller
   than before this branch.

After those fixes, no unresolved correctness, realtime-safety, architecture,
test, or documentation finding remains. Analysis allocates only on the
non-realtime projection path. The callback receives bounded scalar evidence
through the existing coherent atomic snapshot and performs no new allocation,
I/O, locking, or analysis.

No `ActionCommand` was added. Queue, commit, Session/replay, observer, and QA
ownership continue through the existing W-30 resample/damage action path.

## Verification

- focused low-impact selector and synthetic regression tests: pass
- local eight-source development matrix: pass
- exact Beat03 non-listening product preflight: pass
- `riotbox-app` full crate tests after semantic extraction: pass
- `riotbox-audio` full crate tests after semantic extraction: pass
- workspace Clippy with all targets/features and warnings denied after
  semantic extraction: pass
- final `just ci`: pass

## Remaining Gate

RIOTBOX-1425 is technical reachability evidence, not musical quality proof.
After this branch lands, RIOTBOX-1422 must freeze the mechanism, access H22
once through the existing non-listening preflight, and request structured human
listening only for a candidate that passes the unchanged exact-path gates.
