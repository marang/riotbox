# Riotbox Source Graph Spec

Version: 0.1  
Status: Draft  
Audience: MIR/ML, realtime, session, TUI, Ghost

---

## 1. Purpose

This document defines the `Source Graph`, the canonical analysis-derived representation of input material inside Riotbox.

It exists so that:

- the analysis sidecar produces one stable contract
- the realtime core consumes analysis without depending on provider-specific details
- TUI, Scene Brain, capture, and Ghost refer to the same source objects
- session save / load can persist analysis results safely

---

## 2. Core Rule

The Source Graph is the only canonical bridge between source analysis and musical behavior.

No downstream subsystem should depend directly on:

- raw provider outputs
- ad hoc feature blobs
- provider-specific IDs or score names

Provider outputs must be normalized into Source Graph objects first.

---

## 3. Scope

The Source Graph covers:

- source file identity
- timing structure
- section structure
- candidate musical objects
- confidence and provenance
- references needed for replay-safe mutation and capture

It does **not** define:

- session history
- committed actions
- mixer state
- Ghost state
- rendered captures as final persistence objects

Those belong to the session format.

---

## 4. Design Goals

- provider-agnostic
- deterministic enough for replay and session restore
- useful for both MVP and later feral policies
- compact enough for TUI and logging
- explicit about uncertainty

---

## 5. Top-Level Shape

Canonical shape:

```text
SourceGraph {
  graph_version
  source
  timing
  sections
  assets
  candidates
  relationships
  analysis_summary
  provenance
}
```

---

## 6. Source Object

```text
Source {
  source_id
  path
  content_hash
  duration_sec
  sample_rate
  channel_count
  decode_profile
}
```

Rules:

- `source_id` must be stable within the session.
- `content_hash` should identify audio content rather than path alone.
- `decode_profile` should record normalization assumptions that matter for repeatability.

---

## 7. Timing Model

The richer Source Timing Intelligence contract is defined in
`docs/specs/source_timing_intelligence_spec.md`. This section records the
minimum Source Graph v1 timing shape and compatibility fields that existing
consumers still use.

```text
Timing {
  bpm_estimate
  bpm_confidence
  meter_hint
  beat_grid
  bar_grid
  phrase_grid
  hypotheses
  primary_hypothesis_id
  quality
  warnings
  degraded_policy
}
```

The first six fields are the compatibility surface used by existing consumers.
The richer fields are the Source Timing Intelligence surface and should become
the preferred contract for new timing-aware work.

### 7.1 Beat grid

Each beat event should expose:

- beat index
- absolute time
- confidence

### 7.2 Bar grid

Each bar should expose:

- bar index
- start time
- end time
- downbeat confidence
- phrase membership when known

### 7.3 Rules

- uncertain timing must remain visible; do not silently round ambiguity away
- downstream scheduling may use the graph, but realtime commit timing remains a core responsibility
- future timing-aware lanes must consume the shared Source Graph timing contract
  rather than adding lane-local beat-grid or downbeat models

---

## 8. Section Objects

```text
Section {
  section_id
  label_hint
  start_time
  end_time
  bar_start
  bar_end
  energy_class
  confidence
  tags
}
```

Possible `label_hint` values:

- `intro`
- `build`
- `drop`
- `break`
- `verse`
- `chorus`
- `bridge`
- `outro`
- `unknown`

Rules:

- labels are hints, not truths
- sections must be usable even when semantic labeling is weak

---

## 9. Asset Objects

Assets are source-derived regions that may later be sliced, replayed, mined, or promoted.

```text
Asset {
  asset_id
  asset_type
  time_range
  bar_range
  confidence
  tags
  source_refs
}
```

Initial `asset_type` classes:

- `slice`
- `loop_window`
- `hook_fragment`
- `drum_anchor`
- `phrase_fragment`
- `texture_fragment`

---

## 10. Candidate Objects

Candidates are not guaranteed musical truths. They are ranked possibilities for downstream systems.

```text
Candidate {
  candidate_id
  candidate_type
  asset_ref
  score
  confidence
  tags
  constraints
  provenance_refs
}
```

Initial `candidate_type` classes:

- `kick_anchor`
- `snare_anchor`
- `ghost_hit`
- `fill_fragment`
- `loop_candidate`
- `hook_candidate`
- `answer_candidate`
- `capture_candidate`

Rules:

- `score` and `confidence` are distinct
- `score` is consumer-facing usefulness
- `confidence` is analysis certainty

---

## 11. Relationships

The graph must support explicit links between objects.

```text
Relationship {
  relation_type
  from_id
  to_id
  weight
  notes
}
```

Useful relationship types:

- `belongs_to_section`
- `aligns_with_bar`
- `variant_of`
- `supports_break_rebuild`
- `high_quote_risk_with`
- `good_followup_to`
- `capture_parent_of`

These relationships let Scene Brain and feral policy work on graph structure instead of rediscovering it ad hoc.

---

## 12. Analysis Summary

The graph should expose a compact summary for TUI and logs.

```text
AnalysisSummary {
  overall_confidence
  timing_quality
  section_quality
  loop_density
  hook_density
  break_rebuild_potential
  warnings
}
```

This is intentionally shallow. It is a summary surface, not a second graph.

The Feral policy layer may project a read-only scorecard from this graph for
TUI and policy consumers. That scorecard is not a second analysis model; it
must derive from existing graph evidence such as:

- `break_rebuild_potential`
- `HookFragment` assets
- `CaptureCandidate` candidates
- `supports_break_rebuild` relationships
- `high_quote_risk_with` relationships
- analysis warning codes

The current scorecard surface is:

```text
FeralScorecardView {
  readiness
  break_rebuild_potential
  hook_fragment_count
  break_support_count
  quote_risk_count
  capture_candidate_count
  top_reason
  warnings
}
```

Consumers should use this as a compact policy and UX hint. They must still keep
audible behavior behind explicit Action Lexicon, queue / commit, and render
contracts.

`readiness` should be a compact label derived from the same shared Source Graph
evidence contract used by bounded lane consumers, not from separate UI-only
count heuristics.

Current bounded consumers:

- `w30.browse_slice_pool` may use a high or supported Feral scorecard to prefer
  a non-current capture whose `source_origin_refs` match a `CaptureCandidate`
  asset or supported `HookFragment`.
- The selected capture must still be queued and committed through the normal
  W-30 action path.
- TUI surfaces should preserve the reason in compact musician-facing language,
  for example `feral browse cap-03`.
- If the preferred capture changes audible preview material, the slice needs a
  downstream preview or buffer proof, not only a changed log line.
- TR-909 source support may use the same scorecard evidence to lift an
  otherwise neutral `steady_pulse` support profile into `break_lift` when the
  graph has high break-rebuild potential plus supported hook or capture
  evidence.
- That TR-909 bias must remain bounded to the existing source-support render
  policy. It must not create a separate drum arranger, bypass queue / commit
  state, or override explicit `drop_drive` / section-derived break support.
- MC-202 follower / leader render projection may use the same high
  break-rebuild plus supported hook or capture evidence to choose
  `answer_space` hook response outside explicitly hook-like sections.
- That MC-202 bias must remain bounded to existing render-state policy. It may
  create more space for source-backed hooks by reducing note density, but it
  must not create a second phrase generator or bypass role / section semantics.
- If the bias claims musical impact, tests need both render-state proof and an
  audible buffer comparison against the non-Feral control path.

---

## 13. Provenance

Every graph must record where it came from.

```text
Provenance {
  sidecar_version
  provider_set
  graph_generated_at
  source_hash
  analysis_seed
  run_notes
}
```

Rules:

- provider names and versions must be recorded
- if analysis changes materially across versions, the session layer must detect this

---

## 14. Confidence and Uncertainty

Uncertainty must be first-class.

The graph should never imply false precision where the source does not support it.

Minimum uncertainty surfaces:

- tempo confidence
- section confidence
- per-candidate confidence
- explicit warnings for low-confidence graphs

Low-confidence graphs are valid. They should trigger degraded behavior, not graph failure.

---

## 15. MVP Requirements

Source Graph v1 must support:

- one decoded source file
- beat and bar references
- sections
- loop candidates
- drum anchors
- hook or phrase fragment candidates
- summary confidence surfaces
- provenance for session persistence

It does not yet need:

- full harmonic graphing
- cross-source graphs
- deep embedding neighborhoods
- final capture lineage beyond basic references

---

## 16. Consumers

Primary consumers:

- Jam screen
- scheduler and quantization helpers
- TR-909 reinforcement logic
- MC-202 follower generation
- W-30 capture and pad promotion
- Scene Brain
- Ghost suggestions
- session persistence

If a new score or object class is added, a consumer must be named explicitly.

---

## 17. Failure and Degraded Operation

If graph generation is partial:

- timing may succeed while sections remain weak
- sections may exist while hook mining is absent
- loop candidates may be sparse

The graph must still be serializable as long as minimum source identity and timing objects exist.

Warnings should be attached rather than hidden.

---

## 18. Validation Requirements

The Source Graph spec is only useful if validation ties to it.

Required validation:

- schema validation
- deterministic serialization for the same input and analysis seed
- fixture-based sanity checks for candidate density and confidence ranges
- TUI summary rendering from graph-only state

---

## 19. Open Follow-Ups

This draft should be followed by:

1. `Session File Spec`
2. `Audio Core Spec`
3. `TUI Screen Spec`
4. exact serialization format and file location rules
