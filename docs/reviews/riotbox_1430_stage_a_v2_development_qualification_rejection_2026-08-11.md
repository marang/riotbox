# RIOTBOX-1430 Stage-A-v2 Development Qualification Rejection

Date: 2026-08-11
Work class: `audible_vertical_slice` Stage-A gate
Product status: rejected before candidate rendering; no product integration

## Verdict

The one authorized fresh Development-only `StageAQualificationSession`
rejected the frozen Registry-v3 four-source set before candidate rendering.
Two sources passed the unchanged mechanism-blind detector, anatomy, and event
quota. Two did not provide the required minimum of two eligible events, so
their source-feature vectors were undefined without forbidden imputation.
The source-contrast partition gate could not run and the frozen 3-family by
4-source by 2-event matrix was not authorized.

This is a mechanical source/event-admission rejection. It is not a human
hardness or sound-quality verdict and does not say that either rejected source
is bad in another musical role. `quality_proof` and `hardness_proof` remain
false, and `human_verdict` remains `unverified` because no candidate existed
to hear.

## Frozen Inputs

The source-blind stack was committed before source access at Git commit
`7e32036e` and revalidated these raw-byte pins:

- Protocol v2: `b6b35cb14ef34be7f9b7bb6b2bf076ba84842c56914485937f088539e6217878`
- Matrix v3: `0dff59b8d871f75eccd75a5df1ff8080c777f4b76b3559957ce415762b16aa5e`
- Registry v3: `9e5e03ad64319061a4baaa6cee7c40fc5e993171b0d11003ec29767f273bc502`
- executed implementation snapshot:
  `ba6865cc718c188a2084bd1e57b35d3fe16b144f432fd0451a5304c8c549af7f`

No detector equation, anatomy rule, source-contrast equation, threshold,
family topology, ordinal, matrix row, or JSON contract changed after source
access. The source means use the Protocol-v2 exact signed-code i128 sum and
exact-rational-to-binary64 rule; the authoritative bit patterns are bound in
the rejection artifact.

## Access Boundary

Session ID: `7ecb935c-de82-40c9-8045-05f82293014f`

The bounded development-access gate opened exactly the four registered paths
in canonical order and verified each raw WAV SHA-256 and signed RIFF/PCM
format before delivering the same bytes to the qualification owner. It
reported:

- `access_status: completed`
- four files `verified_and_delivered_to_owner`
- `directory_discovery_performed: false`
- `holdout_metadata_comparison.audio_files_opened: false`
- `holdout_audio_accessed: false`
- `commercial_reference_accessed: false`
- `candidate_render_started: false`

The nine active holdout identities participated only in the metadata-level
pre-open exclusion check. No holdout audio, commercial reference, source
directory, rendered candidate, or playback path was accessed.

## Qualification Results

| Development case | Pre-NMS | NMS | Event-level onsets | Resolved bodies | Frozen events | Result |
| --- | ---: | ---: | ---: | ---: | ---: | --- |
| `oga_cinameng_can_be_so_beautiful` | 55 | 41 | 9 | 5 | 3 | qualified |
| `freesound_djericmark_724939` | 31 | 22 | 13 | 13 | 3 | qualified |
| `freesound_cyclez_493560` | 94 | 49 | 15 | 1 | 1 | rejected |
| `freesound_justabeat_458897` | 0 | 0 | 0 | 0 | 0 | rejected |

`freesound_cyclez_493560` produced one eligible body-bearing event. Its other
primary candidates were refused as `physical_onset_unresolved` or
`attack_turnover_unresolved`; one event is below the frozen minimum of two.
`freesound_justabeat_458897` produced no detector peaks and therefore no event
onsets. Both sources received `insufficient_eligible_events` and
`source_feature_requirements_unmet`. The overall refusal is
`positive_source_failed` for those two case IDs.

Because two sources lacked a complete feature vector, the analyzer emitted no
pair contrasts, valid partitions, or bound event catalog. It did not invent
events, impute features, substitute a fallback source, or tune the detector.

## Local Evidence Identity

The bounded session remains local at
`/tmp/riotbox-1430-stage-a-v2.olSYzv`. Exact artifact hashes:

- session JSON:
  `5ca4bcb032e82399932c1e275b1885362a0bef7e0b1952da41affde5e71de326`
- development access log:
  `f892cd4e300c7aec63f345386cc04a8a220b8b1239d4b770ae4dbdb27dcaa0f8`
- qualification rejection:
  `f023736a0c1c7b19668a4497015adffe096fc5cea948135320908789704c9f3d`
- qualification commit marker:
  `96cee5f546e52f7eeee4aeacce7ba84d822616c84ea929c2b64a7e8b7fac3639`

The qualification commit marker is a local evidence marker, not a Git commit.
It binds the final session, access log, and rejection artifact.

## Stop Rule

- Do not execute the frozen 24-condition candidate matrix.
- Do not render a candidate or request human playback.
- Do not tune Protocol v2, Prequalification v3, Detector v1, Anatomy v2,
  source roles, thresholds, ordinals, Registry v3, or Matrix v3 from this
  result.
- Any recomputation requires Protocol v3, the relevant component-version
  bump, a newly frozen legal Development snapshot, and a new decision before
  source access.
