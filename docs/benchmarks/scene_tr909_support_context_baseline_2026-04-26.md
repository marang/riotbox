# Scene TR-909 Support Context Baseline 2026-04-26

- Timestamp: `2026-04-26`
- Base commit: `5064de4`
- Benchmark family: `readability`
- Previous baseline: `None`

## Scope

This is the first bounded readability baseline for the Scene-target TR-909 support context.

It covers one narrow question:

- when Scene Brain projects a landed Scene into TR-909 `SourceSupport`
- can a performer or reviewer tell whether the support profile came from the Scene target or from the transport-bar fallback?

This is not a full transition-engine benchmark. It records how the current diagnostics expose the render-state/profile coupling that landed across `RIOTBOX-230` through `RIOTBOX-233`.

## Current Surfaces

The current support-context labels appear in three places:

- `Log` screen, `TR-909 Render` panel:
  - `render <mode> via <route> | <profile> / <context>`
- `Jam Inspect` TR-909 detail:
  - `profile <profile> | context <context> | route <route>`
- `docs/jam_recipes.md`, Recipe 10:
  - explains what `scene_target` and `transport_bar` mean while trying a Scene jump

## Label Contract

Expected support-context labels:

- `scene_target`
  - TR-909 source support is using the projected source section for the landed Scene target.
- `transport_bar`
  - Riotbox could not map the Scene id to a source section, so TR-909 source support falls back to the current transport bar's source section.
- `unset`
  - TR-909 is not in source-support mode, or no source-support context is active.

These labels are diagnostics for the current render state. They are not a promise of a finished arranger, transition engine, or adaptive drummer.

## Reading Baseline

Starting point:

- at least one projected Scene target exists in the session
- the projected Scene id maps back to a Source Graph section
- TR-909 support rendering is active after the Scene change lands

Expected reading path:

- queue and land a Scene jump
- open `Log`
- read the `TR-909 Render` panel
- confirm the line ends with `/ scene_target` when the profile follows the landed Scene target
- confirm older or unmapped Scene ids still report `/ transport_bar` instead of pretending to have Scene-target context

Expected compact examples:

- `render source_support via drum_bus_support | break_lift / scene_target`
- `render source_support via drum_bus_support | break_lift / transport_bar`
- `render idle via source_only | unset / unset`

## Pass Criteria

- A reviewer can tell which source section is driving TR-909 support without reading code.
- `scene_target` appears only when a Scene target was mapped to source material.
- `transport_bar` remains understandable as the safe fallback.
- `unset` keeps non-source-support modes from looking Scene-coupled.
- The wording matches Recipe 10 and the fixture-backed regression from `RIOTBOX-233`.

## Current Limits

- This baseline is text/readability-only.
- It does not validate audio difference by listening pack.
- It does not add a graphical transition meter or arranger view.
- The current implementation couples render profile selection to Scene/source context; it is still not a finished musical transition engine.

## Follow-Up

- Use this baseline before renaming `scene_target`, `transport_bar`, or `unset`.
- Future TUI simplification should preserve the same readable distinction even if the labels move from `Log` into a more musical visual cue.
- Future audio QA should add a listening baseline once Scene-target support produces enough audible variation to evaluate by ear.
