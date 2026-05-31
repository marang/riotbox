# Riotbox Jam Taste / Proof Glossary

Musician-facing interpretation for the compact P015 Jam taste/proof language.

Scope:

- keep the default Jam screen playable
- explain confidence and proof cues without internal QA jargon
- preserve the deeper proof details on Jam Inspect, Log, Source, and generated
  observer/audio gates

## Taste Cues

- `taste cautious | confirm grid before scene moves`
  - You can keep playing, but do not treat timing-sensitive scene movement as
    trusted yet.
  - Confirm the grid, listen first, or use a simple lane gesture before judging
    the scene behavior.
- `taste scene-ready | trusted grid can steer scene moves`
  - The bounded manual scene path has enough timing trust to steer a scene jump
    or restore.
  - This is confidence language for the next manual move, not proof that Riotbox
    has become an automatic arranger.
- `taste sketch | fallback timing only`
  - Riotbox is staying playable on a safe timing fallback.
  - Treat scene timing as a sketch until stronger source timing evidence exists.
- `taste waiting | needs two scenes`
  - Scene movement is not ready because there is not enough scene material.
- `taste unknown | timing unavailable` or `taste unknown | load source graph`
  - Riotbox cannot currently give useful scene-timing confidence for this run.

## Proof Cues

- `proof none yet | audible moves need output evidence`
  - No landed audible scene movement in this run has proof attached yet.
  - The right next step is to land a move, inspect Log/Jam Inspect, or use the
    relevant proof recipe when the sound feels suspicious.
- `proof pending scene move | wait commit + output`
  - A scene move is queued but not landed.
  - Wait for the commit boundary before judging the result.
- `proof landed movement | inspect replay/audio evidence`
  - A scene movement landed and the deeper inspect path can show the replay and
    output-proof obligations.
  - This does not mean the live host device was recorded; host playback still
    needs local listening when the audible result matters.

## Current Proof Anchors

- `just p014-scene-movement-observer-probe` is the bounded scene movement
  observer/audio correlation gate.
- `just p015-jam-taste-recipe-proof` checks the P015 perform/inspect split and
  reuses the P014 scene movement proof for landed movement evidence.
- Recipe 16 in `docs/jam_recipes.md` is the short user path for reading these
  cues from a real source session.

These gates are QA anchors. They should not make ordinary playing feel like a
test harness.
