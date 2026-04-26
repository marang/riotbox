# Periodic W-30 Capture Seam Review 2026-04-26

Scope:

- focused current-state review of the W-30 Capture / audition / Do Next seam after `RIOTBOX-268` and `RIOTBOX-269`
- reviewed `crates/riotbox-app/src/ui.rs`, `crates/riotbox-app/src/jam_app.rs`, `crates/riotbox-core/src/view/jam.rs`, `docs/specs/tui_screen_spec.md`, and `docs/jam_recipes.md`
- review type is current-state architecture/design review via `review-codebase`, not a diff-only review

## Findings

### 1. Capture Do Next reconstructs W-30 audition pending intent from generic pending actions

- Location: [ui.rs](/home/markus/Dev/riotbox/crates/riotbox-app/src/ui.rs:3628), [jam.rs](/home/markus/Dev/riotbox/crates/riotbox-core/src/view/jam.rs:70), [jam.rs](/home/markus/Dev/riotbox/crates/riotbox-core/src/view/jam.rs:717)
- Category: `scope`
- Severity: `minor`

The Capture `Do Next` surface now correctly explains queued raw and promoted auditions, but it has to scan `JamViewModel.pending_actions` and match action command strings to recover whether the pending cue is raw audition or promoted audition. The core view model only exposes `w30_pending_audition_target`, which is enough for a compact lane cue but not enough for a Capture next-step sentence that also needs command intent and quantization.

Impact:

- the user-facing Capture explanation stays coupled to action-command string names
- future W-30 audition variants could update queue behavior while leaving Capture `Do Next` wording stale
- this repeats the same pending-classification work already partially represented in `LaneSummaryView`

Suggestion:

- add a small view-model projection for W-30 pending audition intent, including kind, target, and quantization
- have Capture `Do Next`, routing diagnostics, and compact lane labels consume that projection instead of re-scanning generic pending actions
- keep this as presentation-model work; do not introduce a second action system

Follow-up: `RIOTBOX-271`

### 2. Capture target kind is still inferred from display strings on the TUI side

- Location: [ui.rs](/home/markus/Dev/riotbox/crates/riotbox-app/src/ui.rs:3606), [ui.rs](/home/markus/Dev/riotbox/crates/riotbox-app/src/ui.rs:3931), [jam.rs](/home/markus/Dev/riotbox/crates/riotbox-core/src/view/jam.rs:382), [jam.rs](/home/markus/Dev/riotbox/crates/riotbox-core/src/view/jam.rs:747)
- Category: `scope`
- Severity: `minor`

`CaptureSummaryView.last_capture_target` is a display string such as `pad bank-b/pad-03` or `scene drop-1`. The TUI then branches on `starts_with("pad ")` and `starts_with("scene ")` to decide whether to show W-30 hit/audition guidance or Scene guidance. This is currently stable because the formatting is local and tested, but it makes display wording carry routing semantics.

Impact:

- a future wording cleanup could accidentally change behavior-level guidance
- Capture `Do Next` and `hear ...` labels can drift if one string formatter changes but the other string-prefix checks do not
- the view model has already lost the typed `CaptureTarget` shape before the TUI gets it

Suggestion:

- project a small typed capture-target view, for example W-30 pad, Scene, other, or unassigned, alongside the existing display label
- keep the display label for text rendering, but use the typed target kind for branching
- preserve current wording and tests while removing the string-prefix dependency

Follow-up: `RIOTBOX-272`

## Boundary Review

- Architecture and boundaries: the seam still uses the existing Source Graph, Session, Action Lexicon, Action Queue, and Jam view projection. No shadow action system or persistence path was introduced.
- Design pattern consistency: the Capture surface follows the existing `Line` / `Span` TUI renderer style and uses the semantic pending-detail helpers added in the recent hierarchy work.
- Technical debt and maintainability: the current risk is presentation-model completeness, not runtime correctness. The TUI has enough information to render correctly today, but it gets some of that information from generic pending-action strings and display-label prefixes.
- Cross-module coupling: `riotbox-app` remains the orchestration/UI layer, but two small view-model gaps cause it to know more about W-30 action command names and capture target formatting than it should.
- Security and performance: no security-sensitive path is involved; the extra scans are small UI work and do not touch the realtime audio callback.

## Review Notes

- No blocker was found for the current W-30 audition guidance.
- The next useful work should stay small and model-facing: first project W-30 pending audition intent into the Jam view, then remove string-prefix target-kind branching from Capture UI.
