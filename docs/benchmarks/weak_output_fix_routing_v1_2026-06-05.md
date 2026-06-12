# Weak-Output Fix Routing v1

`scripts/route_weak_output_fixes.py` maps weak/fail Riotbox audio reports to
concrete production fix categories:

- `source_selection`
- `chop_policy`
- `drum_pressure`
- `bass_movement`
- `mix_bus`
- `destructive_gesture`
- `fixture_threshold`
- `ui_cue`

The router consumes existing agent reviews, human listening labels, destructive
variation reports, source-family professional manifests, and automated musical
fitness manifests. It does not create a new musical judge. It reuses current
failure codes and reason tags, then reports the artifact to hear, strongest
audible element, main weakness, and proposed next fix category.

P023 also projects the routed cases into `production_fix_candidates`. Each
candidate groups one fix category, case ids, source families, artifact refs,
software next step, and musician payoff. These candidates are work selection
inputs for later implementation slices, not proof that the sound is good.

This is a production-actionability diagnostic only:

- `human_verdict` remains `unverified`
- `quality_proof` remains `false`
- `automated_musical_approval` remains `false`
- scripted or negative fixtures cannot be used as product-quality proof
- `production_fix_candidates[]` also keep `quality_proof: false` and
  `automated_musical_approval: false`

Run:

```bash
just weak-output-fix-routing-fixtures
```

The committed fixture manifest proves weak hook/pressure, source-loss,
flat-stutter, hookless tonal material, weak sparse-bass pressure, and
source-masked generated support route to concrete production work.
