# Periodic Scene Brain TUI Seam Review 2026-04-25

Scope:

- focused current-state review of the Scene Brain / Jam TUI seam after `RIOTBOX-204` through `RIOTBOX-207`
- reviewed Scene Brain cue rendering, fixture-backed regression seams, recipe wording, and test fixture taxonomy
- review type is current-state architecture/design review, not a diff-only review

## Findings

### 1. Recipes still document the pre-direction restore-ready cue

- Location: [docs/jam_recipes.md](/home/markus/Dev/riotbox/docs/jam_recipes.md:203), [docs/jam_recipes.md](/home/markus/Dev/riotbox/docs/jam_recipes.md:280), [crates/riotbox-app/src/ui.rs](/home/markus/Dev/riotbox/crates/riotbox-app/src/ui.rs:1391)
- Category: `scope`
- Severity: `minor`

The restore-ready UI now surfaces the energy direction when enough data exists, for example `restore drop/high ready | rise | Y brings back drop/high`. The recipes still teach the older shape without the direction segment. That is not a runtime bug, but it weakens the first-run learning path because users may not know what the new `rise/drop/hold` token means.

Suggestion:

- update the Scene Brain recipe examples to include the optional `| rise/drop/hold |` segment
- explain that the direction is shown only when current and restore energies are both known

### 2. Suggested gestures omit the restore energy direction shown by footer/help

- Location: [crates/riotbox-app/src/ui.rs](/home/markus/Dev/riotbox/crates/riotbox-app/src/ui.rs:1391), [crates/riotbox-app/src/ui.rs](/home/markus/Dev/riotbox/crates/riotbox-app/src/ui.rs:1513), [crates/riotbox-app/src/ui.rs](/home/markus/Dev/riotbox/crates/riotbox-app/src/ui.rs:1840)
- Category: `scope`
- Severity: `minor`

The footer and help overlay now expose the restore energy direction, but the primary suggested gesture line still renders only `[Y] restore <scene> now`. That means the highest-priority action prompt does not carry the same musical intent as the lower-priority explanatory surfaces.

Suggestion:

- add the compact direction to the suggested restore gesture when available, for example `[Y] restore drop now (rise)`
- keep the current fallback when energy direction is unknown

### 3. Scene regression fixture taxonomy is duplicated across three test seams

- Location: [crates/riotbox-app/src/ui.rs](/home/markus/Dev/riotbox/crates/riotbox-app/src/ui.rs:4603), [crates/riotbox-app/src/jam_app.rs](/home/markus/Dev/riotbox/crates/riotbox-app/src/jam_app.rs:1604), [crates/riotbox-core/src/view/jam.rs](/home/markus/Dev/riotbox/crates/riotbox-core/src/view/jam.rs:734)
- Category: `scope`
- Severity: `minor`

The shared Scene regression JSON now depends on a small label-to-energy taxonomy. The same taxonomy is duplicated in app UI tests, app state tests, and core view tests. That is still acceptable for a small fixture, but it creates drift risk: changing fixture semantics requires coordinated edits in three places before the shared fixture can stay trustworthy.

Suggestion:

- centralize the fixture-only Scene label/energy taxonomy in a small shared test helper where practical
- if cross-crate sharing is too heavy, at least document the mapping beside `scene_regression.json` so future fixture edits do not silently diverge

## Review Notes

- No blocking architecture regression was found in the Scene Brain / TUI seam.
- Queue/commit semantics, Session state, and Jam view projection remain on the established product spine.
- The actionable risks are mostly wording drift and test-fixture taxonomy drift, so follow-up tickets should stay small and not become a broad Scene Brain refactor.
