# Riotbox Source Transport Map Capture

## Summary

Riotbox should make source navigation, grid trust, and capture feel like one
musical instrument workflow, not a sample-editor detour.

The target workflow is:

1. load a source
2. hear the source through the transport
3. see an adaptive Source Map
4. audition and confirm the timing hypothesis
5. seek by bar or phrase
6. capture a 1-bar, 4-bar, or phrase-length moment
7. raw-audition the capture
8. promote and hit it through the W-30 path
9. restore the same monitor, timing, transport, and capture state from Session
   and replay

The implementation principle is **Ingenious First**: choose the musician-facing
workflow and durable contracts first, then split the work into reviewable slices.
Do not ship UI-only state that cannot be restored, replayed, heard, or proven.

## Product Contract

`Space` remains transport play / pause. What the musician hears while transport
runs is controlled by a persisted monitor mode:

- `source`: only the decoded source is heard
- `blend`: source and Riotbox lanes are heard together
- `riotbox`: only the generated / performed Riotbox lanes are heard

The default monitor mode after loading a source is `source`, so the first
interaction is hearing the original material while reading the musical map.

The Source Map is an adaptive musical map, not a DAW editor:

- default rendering uses one or two rows of block characters
- the main row shows source energy / loudness
- the second row shows peaks or transient emphasis
- separate rows or markers show bars, playhead, and capture range
- colors may improve scanning but must not carry the only meaning
- Ratatui `Canvas` remains a planned dense Source-screen alternative once the
  first block-map contract is stable

The map is musical when the timing contract is usable and honest when it is not:

- usable grid: compute energy from source time, project it through the selected
  beat/bar timing hypothesis, and group it visually by bar
- unusable grid: use a time-uniform fallback and do not imply precise bar capture
- phrase and section labels are secondary context, not the primary grid
- semantic section labels such as `drop`, `break`, or `chorus` are shown only
  when Source Graph evidence explicitly supports them; otherwise use neutral
  labels such as `section A`, `section B`, and `section C`

Grid confirmation is a real product state:

- the musician confirms the currently selected timing hypothesis after source
  audition
- confirmation is represented by action, session, replay, observer, and log
  surfaces
- confirmation can be undone or reverted
- confirmation is not a local UI flag and must not silently rewrite Source Graph
  timing evidence

Navigation uses the transport, not a separate editor cursor:

- `Left` / `Right` seek to previous / next bar
- `Up` / `Down` seek to previous / next phrase
- seek preserves the current play / pause state
- source bounds clamp by default; looping or wrapping is a later explicit mode

Capture defaults to musical moments rather than exact manual ranges:

- default capture length is `4 bars`
- selectable lengths are `1 bar`, `4 bars`, and `phrase`
- `1 bar` and `4 bars` start at the next bar
- `phrase` starts at the next phrase when phrase evidence is usable
- when phrase evidence is not usable, phrase capture falls back visibly to
  `4 bars`
- `o` raw-auditions the latest committed capture before promotion

## Implementation Slices

1. Document this plan and mirror the stable contracts into the roadmap and specs.
2. Add monitor mode and grid-confirm contracts to Action, Session, replay, and
   observer surfaces.
3. Add realtime-safe source monitor playback and monitor mix presets.
4. Add the adaptive Source Map model and TUI surfaces.
5. Add bar / phrase seek controls through transport seek.
6. Add capture-length selection and phrase fallback.
7. Add the musician recipe and end-to-end QA proof for hear / see / capture,
   replay / restore, and visual accuracy.

## QA Contract

The acceptance order is:

1. hear / see / capture: source monitor is audible, the map shows position, seek
   moves the heard source, capture lands, and raw audition plays the captured
   moment
2. replay / restore fidelity: monitor mode, confirmed grid, transport position,
   capture length, and capture result restore deterministically
3. visual accuracy: map rows, playhead, bars, capture range, grid fallback, and
   monochrome readability are covered by focused snapshots

Audio proof must show that `source`, `blend`, and `riotbox` monitor states are
not silently collapsed into one another, and that source seek changes the audible
source excerpt without realtime file I/O.
