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
