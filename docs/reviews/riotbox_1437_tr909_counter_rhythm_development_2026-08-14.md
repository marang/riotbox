# RIOTBOX-1437 TR-909 Counter-Rhythm Development Qualification

Date: 2026-08-14  
Partition: Development only  
Holdout access: none  
Commercial-reference access: none

## v1 Frozen Result

`tr909_counter_rhythm_slam_v1` was implemented and validated source-blind at
commit `7a4b5cbf`. Core policy, atomic callback projection, exact RuntimeMix
parity, source/path independence, fixed event count, two accent/two donor
slots, multiplier-sum equality, kick invariance, and downbeat/backbeat
invariance passed synthetic tests before Development audio access.

A fresh bounded Development session then opened only the registered mandatory
dense case, `dense_beat03_130`. The analyzer produced trusted phrase evidence
with confidence `0.916584` and transient density `1.0`, but offbeat-onset
density `0.125`. The frozen v1 activation minimum was `0.25`; therefore the
typed policy correctly refused the counter-rhythm. No candidate or human-review
audio was generated. Because the dense case was mandatory, the session stopped
fail-closed before either remaining Development file was opened.

Result: `v1_terminal_fail_closed_pre_candidate`.

## v2 Preregistration Boundary

The v1 result exposed a modeling error rather than evidence for a different
gain constant: offbeat density was used both as a policy selector and as an
eligibility gate. For H-PATTERN-1, a transient, low-offbeat source can be the
case that most clearly calls for an eighth-note answer; refusing it prevents
the intended anchor/counter-role experiment from running at all.

`tr909_counter_rhythm_slam_v2` therefore keeps every renderer constant, role,
event-count invariant, source trust gate, transient-density gate, phase
control, technical threshold, and human rejection rule from v1. It changes
only the selector domain: finite offbeat density in `[0.0, 0.55)` selects
`eighth_answer`, while density at least `0.55` selects
`late_sixteenth_pickup`. This new mapping is frozen in
`docs/benchmarks/tr909_counter_rhythm_slam_development_v2.json` under RBX-290
before any remaining Development source is opened or any v2 result is
computed.

The v2 run remains Development-only and may support a bounded musician taste
decision, not a holdout, release-readiness, or `percussive_hard` claim.

## v2 Frozen Result

After the first-phrase harness alignment was corrected without changing policy
or audio constants, the mandatory dense case selected `eighth_answer` and
rendered the exact candidate and phase control. Both paths had zero pre-limiter
clips, zero limited samples, and zero post-limiter clips. Their maximum
step-local delta RMS was `0.0145695`, below the frozen `0.02` gate. The v2
session therefore stopped before opening either remaining Development source
and generated no review WAV.

Result: `v2_terminal_fail_closed_pre_review`.

## v3 Preregistration Boundary

v2 showed that the selected positions reach the exact product path but its
`1.5`/`0.5` hierarchy remains too small for the preregistered audibility gate.
`tr909_counter_rhythm_slam_v3` changes the musical topology rather than lowering
that gate: the same two donor Snare/Hat positions become actual local holes
(`0.0`), and the same two counter-role positions receive their conserved weight
(`2.0`). This combines H-PATTERN-1's answer role with local negative space.

Every source selector, trust gate, kick/anchor promise, affected slot, event
grid, multiplier sum, exact-mix threshold, limiter rule, and human rejection
rule remains unchanged. The new constants and seed `14370003` are frozen in
`docs/benchmarks/tr909_counter_rhythm_slam_development_v3.json` under RBX-291
before any v3 audio access.

## v3 Frozen Result

The bounded v3 session opened exactly the three registered Development files
and no Holdout or commercial-reference audio. Results were:

| Case | Policy/result | Maximum step-local delta RMS | Limiter |
| --- | --- | ---: | ---: |
| `dense_beat03_130` | `eighth_answer` / pass | `0.0278752` | `0` |
| `tonal_rusharp_120` | typed refusal: resolved tonal live policy is not `MainlineDrive` | n/a | n/a |
| `sparse_kicksnr_120` | `eighth_answer` / fail | `0.0195691` | `0` |

The technical contract required the dense case plus at least two qualified
cases. Only dense passed. Sparse remained below the unchanged `0.02` floor,
and tonal correctly stayed outside a mainline drum-owner gesture. Therefore v3
stopped fail-closed before writing review WAVs or requesting human listening.

Result: `v3_terminal_fail_closed_pre_review`.

## Product Consequence

The counter-rhythm implementation was removed after qualification. The branch
retains only the immutable v1-v3 contracts, RBX-289 through RBX-292, and this
negative Development record. No counter-rhythm Slam behavior or quality claim
enters Riotbox.

The experiment still narrowed the next causal question: callback-safe
Snare/Hat weight redistribution, even when it creates two complete donor holes,
does not transfer robustly across the required source set at the established
product level. A future audible slice should change the rendered material or
voice articulation itself, or use source-backed arrangement/chop ownership;
it must not create v4 by lowering `0.02` or moving the same four multipliers.
