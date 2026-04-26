# Riotbox TUI Screen Spec

Version: 0.1  
Status: Draft  
Audience: TUI, product, Ghost, realtime

---

## 1. Purpose

This document defines the TUI screen model for Riotbox, with MVP emphasis on the `Jam` experience.

It exists so that:

- the interface reflects the product spine instead of exposing raw internals
- realtime control, trust, and visibility stay balanced
- Ghost and analysis state appear as explainable support rather than clutter

---

## 2. Core Rule

The TUI is an instrument surface, not an inspector dump.

The user should always be able to answer:

- what am I hearing
- what is about to happen
- what can I do next
- what just changed

If the screen cannot answer those questions quickly, it is too abstract or too noisy.

---

## 3. Screen Philosophy

Riotbox should follow the masterplan's layered UX:

- `Jam`: immediate live control
- `Sculpt`: deliberate shaping and capture
- `Lab`: deeper inspection and diagnosis

The MVP only requires the `Jam` screen as fully real.

Supporting screens may remain partial until the action, session, and graph contracts stabilize.

---

## 4. Screen Set

Initial screen set:

- `Jam`
- `Source`
- `Capture`
- `Log`
- `System`

Priority order:

1. `Jam`
2. `Log`
3. `Source`
4. `Capture`
5. `System`

---

## 5. Jam Screen

The Jam screen is the core performance surface.

It must expose:

- current transport state
- current scene or section context
- macro state
- lane summaries
- pending actions
- recent committed actions
- key confidence or warning signals
- a compact timing rail for the next queued live gesture
- suggested Scene launch gestures that may name the next target and energy `rise`, `drop`, or `hold`, or explain that launch waits for more scene material
- Scene jump availability should come from the shared Jam view model, not be re-inferred separately by individual TUI surfaces
- when section energy is known, the shared Scene launch target may prefer the first deterministic contrast candidate over the immediately adjacent same-energy scene
- restore-ready Scene cues that name the target and, when known, whether restore is an energy `rise`, `drop`, or `hold`
- post-commit Scene cues that keep a readable monochrome sentence while visually separating the live scene/result, restore target, compact support hints such as `909 lift`, and next perform keys

### 5.1 Jam goals

- playable in seconds
- readable at a glance
- safe for live use
- not flooded by low-level parameters

### 5.2 Jam regions

Recommended regions:

1. transport and source summary
2. scene and section strip
3. macro strip
4. lane status row
5. pending / recent actions
6. Ghost suggestion area
7. warning / health footer

---

## 6. Source Screen

The Source screen should expose analysis-derived structure without becoming a forensic workstation.

Show:

- source identity
- tempo / confidence
- section list
- loop and hook candidate summaries
- Source Graph warnings

This screen helps build trust when the Jam surface behaves unexpectedly.

---

## 7. Capture Screen

The Capture screen should center capture as a musical workflow.

Show:

- armed or recent captures
- target pad or bank
- capture provenance summary
- promotion results
- favorite or pinned captures
- the audible handoff path from stored capture to raw audition, promoted pad, hit, promoted audition, or recall
- a primary next-step cue for capture, promote, hit, and audition before lower-level routing diagnostics

MVP rule:

- this may begin as a focused summary screen, not a deep sampler editor
- the screen must not imply that a stored capture is already heard; if a promotion or audition step is still required, say so directly
- first-action cues should outrank provenance and routing details on the default Capture surface
- first-action cues should prefer pending capture or promotion intent over the last committed capture state when a capture-path action is queued
- pending `Do Next` cues should carry semantic pending emphasis distinct from committed fallback guidance
- secondary pending-capture panels should show the most relevant queued capture-path action first and send overflow detail to `Log`
- contextual help may point to `Do Next`, `hear ...`, and `Log` confirmation, but should not duplicate the full Capture screen

---

## 8. Log Screen

The Log screen is where action trust becomes visible.

Show:

- accepted actions
- queued actions
- rejected actions
- Ghost suggestions and accept / reject outcomes
- important warnings

Rules:

- action wording must use the Action Lexicon
- timestamps and quantization targets should be visible
- log noise must be controlled
- TR-909 render diagnostics may show compact support context and accent cues, but must remain read-only diagnostics rather than new controls
- `accent scene` means `scene_target` source support is getting the bounded Scene-target support accent
- `accent off fallback` means source support fell back to `transport_bar` and no Scene-target accent is active
- non-source-support modes should keep accent wording in an `off` state
- Jam Inspect may show the same TR-909 profile / context / accent / route tuple for diagnosis, but the primary Jam surface should not require reading this tuple to play
- the accent cue is not a transition-engine promise; it only names the current render diagnostic state

---

## 9. System Screen

The System screen should expose operational confidence, not just debug trivia.

Show:

- audio health metrics
- sidecar status
- session file status
- current versions or schema status where relevant
- degraded-mode warnings

---

## 10. Information Hierarchy

Highest-priority information:

1. audible state
2. imminent committed change
3. available live controls
4. important confidence or failure warnings
5. history and diagnostics

This ordering should control layout decisions and keybinding design.

### 10.1 First Terminal Emphasis Tokens

The first color/emphasis layer should stay small and semantic:

- cyan + bold for primary perform controls
- yellow + bold for active Scene timing or restore affordances
- red + bold for warning labels, with yellow warning detail
- green for clear/healthy confirmation
- dark gray for lower-priority status diagnostics

For Scene post-commit cues:

- the current live scene/result may use green + bold as a positive landing confirmation
- restore targets and compact Scene-support hints such as `909 lift` may use yellow without becoming new controls
- next-action key tokens should use the same cyan + bold treatment as other primary perform controls
- labels and separators should stay low-emphasis so the cue reads as an instrument line, not as a diagnostic tuple

For queued timing rails:

- countdown glyphs such as `[===>]` may use yellow + bold as the active snap-point cue
- boundary labels such as `next bar` or `next phrase` may use the same yellow + bold emphasis when the rail is waiting on that boundary
- transport counters such as beat, bar, and phrase indices should stay low-emphasis context rather than competing with the snap point
- this hierarchy does not imply new scheduler behavior or a separate timing visualization widget

For pending Scene intent cues:

- the pending Scene verb may use yellow + bold to mark that a live gesture is armed
- target Scene ids and boundary labels should be visually scannable without adding diagnostic fields
- energy direction such as `energy rise`, `energy drop`, or `energy hold` may use green + bold when it confirms the musical direction of the queued move
- the line should still read as one monochrome sentence, for example `launch -> scene-02-drop @ next bar | energy rise`

For latest-landed result cues:

- the committed command may use green + bold to mark that the queued action actually landed
- Scene energy direction may use the same green + bold treatment when it confirms the resulting musical direction
- actor labels and separators should stay low-emphasis context
- the line should remain readable as plain text, for example `landed user scene jump | energy rise`

Do not use color as the only carrier of meaning. The text must still read correctly in monochrome snapshots and low-color terminals.

---

## 11. Interaction Rules

The TUI must support:

- fast macro adjustment
- scene launch or restore
- capture triggering
- action approval or rejection for Ghost suggestions
- lock visibility

Rules:

- destructive or identity-changing actions should surface pending state before commit
- scene launch suggestions should preserve a generic `[y] jump` fallback when the next target or energy direction is unknown, but may say `[y] jump waits for 2 scenes` when the view knows no queueable Scene jump exists yet
- the known no-queueable-Scene case should remain a bounded view-model availability state so `Jam`, footer, and help wording cannot drift independently
- scene restore affordances should preserve a target-only fallback when current or restore energy is unknown, but should include `rise/drop/hold` wherever both sides are known
- the user must be able to tell whether an action is immediate, queued, or committed
- no screen should imply committed state when the action is still pending

---

## 12. Ghost Visibility

Ghost should appear as bounded assistance.

Required visibility:

- current mode
- current suggestion if present
- accept / reject affordance
- explanation or short rationale
- lock or budget blockers when relevant

Ghost must not occupy the center of the screen when the user is actively performing.

---

## 13. Confidence and Warning Surfaces

The TUI should expose uncertainty honestly.

Examples:

- weak tempo confidence
- sparse loop candidates
- degraded sidecar status
- capture failure
- replay mismatch warning

Warnings should be legible but not panic-inducing.

---

## 14. MVP Requirements

TUI v1 must provide:

- a usable Jam screen
- visible macro state
- visible scene or section context
- visible pending and recent actions
- visible Ghost watch / assist suggestions
- at least one place to inspect analysis confidence
- at least one place to inspect system health

It does not yet need:

- deep editor widgets for every lane
- exhaustive keyboard customization
- full-screen source graph visualization
- complex arrangement editing

---

## 15. Validation Requirements

Required validation:

- screen state reflects committed runtime state
- pending actions render distinctly from committed state
- Ghost suggestion visibility remains understandable under load
- degraded analysis and audio-health warnings render correctly

Workflow tie-ins:

- Track to Jam
- Capture
- Save and restore
- Assisted mutation

---

## 16. Open Follow-Ups

This draft should be followed by:

1. exact Jam layout sketch
2. keybinding map
3. event-to-view-model contract
4. TUI component ownership and state boundaries
