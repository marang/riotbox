# Automated Musical Fitness v1

`riotbox.automated_musical_fitness.v1` is a deterministic rejection gate for
generated Riotbox audio QA artifacts. It can reject silent, clipped,
fallback-collapsed, source-fake, static, lane-imbalanced, or beat-weak output.
It is not human taste approval.

Required top-level report fields:

- `technical_status`
- `automated_musical_fitness_status`
- `result`
- `selected_candidate`
- `failure_codes`
- `score_breakdown`
- `human_verdict: unverified`

The validator reads existing manifest metrics and selects the best passing
candidate when a showcase directory contains multiple candidate manifests. A
passing report means the automated checks did not catch a known bad-output
mode. It does not mean the hook is strong, the sound is exciting, or a musician
has approved the result.

## Deterministic Fixture Corpus

Run:

```bash
scripts/validate_automated_musical_fitness_fixtures.sh
```

Current positive fixture families:

- `valid`: compact source-reactive baseline.
- `valid_break_low_drive`: beat-forward break with strong low-end pressure.
- `valid_tonal_hook_chop`: hook-led chopped source behavior.
- `valid_sparse_bass_pulse`: sparse source with bass-pressure response.

Current negative fixture families:

- `invalid_static`: rejects static loop movement.
- `invalid_source_masked`: rejects generated support masking the source.
- `invalid_source_fake`: rejects non-source-derived contour evidence.
- `invalid_weak_low_transient`: rejects weak low-end and transient pressure.
- `invalid_identical_response_across_sources`: rejects identical response
  signatures for distinct source cases.
- `invalid_fallback_collapse`: rejects fallback-collapsed output.
- `invalid_grid_drift`: rejects weak source-grid alignment.

These fixtures are automated-fitness evidence only. They are deliberately named
as validation fixtures, not as listening approval examples.
