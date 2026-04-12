# Riotbox Preset and Style Spec

Version: 0.1  
Status: Draft  
Audience: product, realtime, TUI, Ghost, feral profile work

---

## 1. Purpose

This document defines how Riotbox expresses presets, profiles, and style behavior.

It exists so that:

- musical character can change without changing core architecture
- the feral profile remains a policy layer instead of a fork
- macros, scoring, and scene tendencies can be bundled coherently
- users and Ghost can target named behaviors safely

---

## 2. Core Rule

Style lives in policy, weighting, macro defaults, and constraints.

Style must not:

- require a separate session format
- require a parallel engine tree
- introduce hidden command semantics
- bypass Source Graph, Action Lexicon, or Audio Core contracts

If a style needs new architecture, it is no longer just a style.

---

## 3. Vocabulary

For this project:

- `preset` means a named bundle of parameter defaults
- `profile` means a broader behavioral policy layer
- `style` means the musical identity expressed through those bundles and policies

Practical rule:

- a preset is usually smaller and more local
- a profile may influence multiple subsystems at once

---

## 4. Style Surfaces

Style may influence:

- macro defaults and ranges
- scoring weights
- scene tendencies
- capture promotion policy
- mutation density
- damage / dirt preferences
- Ghost suggestion bias

Style may not directly redefine:

- action object shape
- session schema
- source graph schema
- realtime thread rules

---

## 5. Preset Object

Canonical shape:

```text
Preset {
  preset_id
  preset_type
  display_name
  description
  target_scope
  parameter_defaults
  parameter_limits
  tags
}
```

Possible `preset_type` values:

- `macro`
- `lane`
- `scene`
- `mix`
- `capture`

---

## 6. Profile Object

Canonical shape:

```text
Profile {
  profile_id
  display_name
  description
  macro_bias
  scoring_weights
  scene_policy
  capture_policy
  ghost_bias
  constraints
  tags
}
```

Rules:

- a profile may point to presets
- a profile may modify weights and tendencies
- a profile may not create hidden unlogged actions

---

## 7. Core Parameter Domains

The style system should work on explicit domains.

Minimum domains:

- source retain
- chaos or mutation pressure
- MC-202 touch
- W-30 grit
- TR-909 slam
- scene aggression
- capture eagerness
- dirt / room intensity

These domains should be stable enough for TUI, Ghost, and presets to share names.

---

## 8. Scene Policy Surface

Profiles may shape scene behavior through policy, for example:

- preferred scene transitions
- strip / build / slam tendency
- variation density
- restore aggressiveness
- repetition tolerance

Rules:

- scene policy biases decisions
- scene policy does not replace Scene Brain logic

---

## 9. Scoring Weight Surface

Profiles may bias scoring outputs, for example:

- break rebuild preference
- hook-fragment preference
- quote-risk conservatism
- capture-worthiness threshold
- damage appetite

Rules:

- the scoring system stays shared
- profiles only adjust weights, thresholds, and priorities

---

## 10. Capture Policy Surface

Profiles may shape:

- which moments are promotion-worthy
- whether captures favor hooks, loops, or abuse bounces
- how aggressively internal resamples are reused

This is especially important for the W-30 path and feral workflows.

---

## 11. Ghost Bias Surface

Profiles may bias Ghost toward:

- more conservative suggestions
- more aggressive scene mutations
- more capture-forward behavior
- stronger restore discipline

Rules:

- Ghost bias influences suggestion selection
- it does not grant extra permissions

---

## 12. Feral Rebuild Profile

`feral_rebuild` is the first major profile expected by the current planning docs.

It should be implemented as:

- profile-level scoring bias
- profile-level scene tendency
- profile-level capture policy
- profile-level dirt and damage appetite
- optional preset bundles for fast activation

It must **not** be implemented as:

- a separate product mode with separate architecture
- a second Ghost system
- a separate persistence model

---

## 13. TUI Requirements

The TUI should expose:

- current active profile
- current relevant preset if any
- profile-relevant macro emphasis
- warnings if a profile is constrained by locks or low-confidence analysis

The user should be able to tell what style logic is active without reading internal config files.

---

## 14. Session Requirements

Sessions must persist:

- active profile ID
- active preset references where relevant
- any profile-adjusted macro state that affects restore

Sessions should not persist style identity only as ambiguous raw parameter blobs if the named profile matters to replay semantics.

---

## 15. MVP Requirements

Preset / Style v1 must support:

- named preset objects
- one active profile at a time
- `feral_rebuild` as a profile layer
- profile-aware Ghost suggestion bias
- profile-aware scoring and scene tendencies
- session persistence of active profile identity

It does not yet need:

- profile blending
- marketplace-style preset packs
- per-user profile inheritance trees
- procedural preset generation

---

## 16. Validation Requirements

Required validation:

- activating a profile changes only approved policy surfaces
- preset recall is deterministic
- profile identity survives save / load
- Ghost suggestion shifts remain within allowed policy boundaries
- feral-specific tests can prove that the profile affects outcomes measurably

---

## 17. Open Follow-Ups

This draft should be followed by:

1. exact preset file format
2. exact profile file format
3. profile-to-score mapping table
4. first concrete `feral_rebuild` policy defaults
