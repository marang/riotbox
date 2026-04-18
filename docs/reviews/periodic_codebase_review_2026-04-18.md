# Periodic Codebase Review 2026-04-18

Scope:

- repo root review after `RIOTBOX-107` through `RIOTBOX-111`
- primary focus on the current Scene Brain seam and the perform-facing shell that now exposes scene launch, restore, and energy cues
- review type is current-state architecture/design review, not a diff review

## Findings

### 1. Scene energy is still a TUI-derived guess instead of an explicit scene contract

- Location: [crates/riotbox-app/src/ui.rs](/home/markus/Dev/riotbox/crates/riotbox-app/src/ui.rs:2332), [crates/riotbox-core/src/view/jam.rs](/home/markus/Dev/riotbox/crates/riotbox-core/src/view/jam.rs:277), [crates/riotbox-app/src/jam_app.rs](/home/markus/Dev/riotbox/crates/riotbox-app/src/jam_app.rs:902)
- Category: `scope`
- Severity: `major`

`current_scene_energy_label(...)` currently sorts source-graph sections and then matches the active scene by index inside `scene_state.scenes`. That lets the shell show one bounded energy cue quickly, but it also means the cue is only correct while scene ordering happens to mirror sorted source sections. The core `JamViewModel` only exposes `active_scene` and `scene_count`, so the Scene Brain energy summary is not yet a shared, explicit contract for non-TUI consumers or for future scene policies that may decouple scenes from simple section order.

Suggestion:

- promote the current-scene energy summary into the app/core presentation model instead of deriving it in the TUI
- represent the scene-to-section or scene-to-energy mapping explicitly enough that future scene generation can evolve without the shell guessing by position

### 2. Perform-facing gesture vocabulary is duplicated across multiple shell surfaces

- Location: [crates/riotbox-app/src/ui.rs](/home/markus/Dev/riotbox/crates/riotbox-app/src/ui.rs:221), [crates/riotbox-app/src/ui.rs](/home/markus/Dev/riotbox/crates/riotbox-app/src/ui.rs:1136), [crates/riotbox-app/src/ui.rs](/home/markus/Dev/riotbox/crates/riotbox-app/src/ui.rs:1221), [crates/riotbox-app/src/ui.rs](/home/markus/Dev/riotbox/crates/riotbox-app/src/ui.rs:1621)
- Category: `scope`
- Severity: `minor`

The perform-facing wording introduced by the recent Jam cleanup now exists in several parallel places: status messages during key handling, footer copy, help overlay copy, and action-summary label mapping. That is workable while the shell vocabulary is still moving quickly, but it creates a growing drift risk: one UX wording change now wants coordinated edits across multiple literal-string tables plus their snapshot tests.

Suggestion:

- centralize perform-facing gesture names and short action labels behind one shell vocabulary table or helper layer
- have status, footer, help, and action-summary renderers reuse that shared wording instead of carrying separate string literals

## Review Notes

- The current repo state does not show a broad new architecture break. The active risk is more specific: Scene Brain is becoming visible on the shell faster than it is becoming an explicit core presentation contract.
- The first finding is the only one that should shape near-term Scene Brain tickets. The second is cleanup pressure that can remain bounded while the shell copy continues to settle.
