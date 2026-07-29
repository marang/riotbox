# RIOTBOX-1427 W-30 Hard Intent Review

Date: 2026-07-29  
Owner: Owner  
Phase: P023 / Controlled Expansion  
Work class: `contract_enabler`  
Direct audible follow-up: RIOTBOX-1422 H25 exact live-path W-30 Hard attempt

## Outcome

The performer now owns the requested W-30 Hard domain before source analysis
selects a realizable policy:

| Committed intent | Suitable source evidence | Product outcome |
| --- | --- | --- |
| `impact` | transient/body policy | `realized_impact` / `source_transient_chop` |
| `impact` | continuous texture policy | `source_mismatch` / Base preserved |
| `texture` | any suitable source audio | `realized_texture` / `source_texture_bite`, no trigger grid |
| either | unsuitable or missing source | `source_unavailable` / Base preserved |
| historical generic params | suitable source audio | `legacy_auto` / historical policy selection |

This prevents valid continuous damage from being mislabeled as a harder beat.
It does not make the current hit shaper harder, change its DSP, or lower its
`0.18`, `1.40`, or `0.30` source-ownership gates.

## Product-Spine Accounting

1. Queue: the current `D` gesture queues typed `impact`; the explicit queue API
   also accepts `texture`. Both use the existing
   `w30.apply_damage_profile` command and next-bar quantization. This enabler
   does not add a second audible TUI gesture before H25.
2. Commit / side effects: capture ID, intensity, and intent remain typed;
   W-30 focus and grit continue through the existing side-effect path.
3. Session / replay: `ActionParams::W30DamageProfile` persists and roundtrips
   the intent. Old `Mutation` actions remain supported as `legacy_auto`; new
   typed actions cannot queue or replay that compatibility-only value.
4. User / observer: RuntimeView, observer Hard detail, and compact TUI
   diagnostics expose requested intent and realized outcome.
5. QA: unit, queue/commit, replay, observer, serialization, fail-closed policy,
   and local multi-source development-matrix checks cover the contract.

No second action, persistence, replay, arrangement, Ghost, or Feral system was
introduced. The callback continues to consume only the realized typed policy;
source analysis and intent resolution remain outside realtime rendering.

## Development Matrix

The existing ignored RIOTBOX-1423 local matrix was extended to project both
intents across eight registered sources and eight labels spanning
golden-path/dense, sparse drums, tonal riff, pad/noise, weak, and bad-timing
material.

- Beat03, Bertsz, Cinameng, Marwan, and Fupi retain transient policy.
- The ambient pad retains texture policy.
- Weak synth and bad-timing water remain unavailable.
- `impact` realizes only on transient policy; texture-classified material
  reports mismatch.
- `texture` realizes on suitable material with zero trigger mask and zero
  source-onset cursor ownership.
- Repeated automatic projection remains deterministic and the existing
  cross-source selection signatures remain diverse.

No H25 holdout, commercial reference source, candidate WAV, or listening
artifact was accessed or generated.

## Human Listening

`human_verdict: unverified`

This slice changes ownership and failure semantics, not the frozen sound
recipe. Replaying prior audio would not provide new musical evidence. Human
listening resumes only when RIOTBOX-1422 H25 produces a technically eligible
exact live-path candidate.
