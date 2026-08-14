# `RIOTBOX-1434` P023: Validate natural velocity controls and freeze multi-cue transfer directions

- Ticket: `RIOTBOX-1434`
- Title: `P023: Validate natural velocity controls and freeze multi-cue transfer directions`
- Linear issue: `https://linear.app/riotbox/issue/RIOTBOX-1434/p023-validate-natural-velocity-controls-and-freeze-multi-cue-transfer`
- Project: `P023 | Sound Excellence / Production Quality`
- Milestone: `M4 | Controlled Expansion`
- Status: `Done`
- Created: `2026-08-11`
- Started: `2026-08-11`
- Finished: `2026-08-11`
- Branch: `feature/riotbox-1434-natural-velocity-controls`
- Linear branch: `feature/riotbox-1434-p023-validate-natural-velocity-controls-and-freeze-multi-cue`
- Assignee: `Markus`
- Labels: `review-followup`, `Spike`, `Analysis`, `Audio`
- PR: `#1389 (https://github.com/marang/riotbox/pull/1389)`
- Merge commit: `0df725b2f60e92ec72158b24a7cfbe842e7086d3`
- Deleted from Linear: `2026-08-14`
- Verification: `Both frozen contract validators and all 21 fail-closed mutation fixtures passed; Python compilation, repository validation, branch review, local just ci, and GitHub Rust CI passed.`
- Docs touched: `docs/benchmarks/percussive_force_natural_velocity_controls_v1.json; docs/benchmarks/percussive_force_natural_velocity_controls_v2.json; docs/reviews/riotbox_1434_natural_velocity_control_qualification_2026-08-11.md; docs/research_decision_log.md`
- Follow-ups: `RIOTBOX-1435 canceled because this ticket produced no source-general directional handoff.`

## Why This Ticket Existed

F4 showed that moving only low-body decay was mechanically valid but
perceptually ineffective. RIOTBOX-1434 therefore tested six already registered
natural-dynamic controls as a narrow directional sanity check before Riotbox
could design a coupled attack, body, resonance, and brightness mechanism.

## What Shipped

- Froze exact v1 identities, hashes, access order, analysis equations, human
  protocol, and claim boundaries before opening any control audio.
- Preserved the v1 pre-open path refusal, then versioned only the documented
  local-audio-root correction in v2 without searching or substituting files.
- Read exactly six registered Philharmonia controls through a bounded access
  log and computed the frozen attack, decay, body-resonance, and brightness
  measurements.
- Built and technically verified one exact blinded primary/reversed artifact.
  In both orders the fortissimo snare was perceived as more forcefully struck;
  the whip produced no meaningful perceived-force difference.
- Recorded the terminal fail-closed result without fitting a threshold,
  selecting an algorithm, or promoting snare-only observations into a
  source-general rule.

## Bounded Outcome

- The technical directions disagreed across controls: whip attack, decay, and
  resonance were non-monotonic, while its monotonic brightness direction was
  opposite to the snare.
- The human result corroborated the snare ordering but not the whip. Under the
  preregistered complete-control rule, this is valuable negative evidence and
  not an implementation handoff.
- No Development, Holdout, or commercial-reference audio was accessed. No
  candidate or product-path audio was rendered.
- RIOTBOX-1428 Stage B remains blocked. Future work requires a genuinely new
  research basis or source-blind causal hypothesis.

## Links

- [Natural-velocity control qualification](../../reviews/riotbox_1434_natural_velocity_control_qualification_2026-08-11.md)
- [Frozen v2 control contract](../../benchmarks/percussive_force_natural_velocity_controls_v2.json)
