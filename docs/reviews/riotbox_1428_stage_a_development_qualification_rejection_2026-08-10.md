# RIOTBOX-1428 Stage-A Development Qualification Rejection

Date: 2026-08-10
Work class: `audible_vertical_slice` Stage-A gate
Product status: rejected before candidate rendering; no product integration

## Verdict

The first fresh development-only `StageAQualificationSession` rejected the
frozen four-source pack before candidate rendering. Two of four positive
sources did not satisfy the frozen mechanism-blind detector and event-anatomy
requirements. The source-contrast catalog therefore remained undefined
without imputation, and the preregistered 3-family by 4-source by 2-event
matrix was not authorized.

This is a mechanical source/event-admission rejection. It is not a musical
hardness verdict, does not say that either source is weak, and cannot award or
deny `percussive_hard` perceptually. `quality_proof`, `hardness_proof`, and the
human verdict all remain false or unverified.

## Frozen Inputs

The session revalidated the exact RBX-252 raw-byte pins before access, before
analysis, and before serialization:

- protocol: `35091e697cacb3c187f9a33f4f41ac85aba26832a4214bf3251dfc703edad840`
- matrix: `aba846138246c95b1c3e5e1973e77bdaa41ce971f799dadadba8edc160967fd6`
- registry: `af98af67d5b0ef9f8478bf800438b268af2a4640bed29d8ec7c87fa585eb6812`
- implementation snapshot:
  `8285107a8c396c7fcbfd52cbe24e3dc8c3a108c56a57487cdb178977a5b2de94`

No detector equation, event-anatomy rule, threshold, family topology,
partition, matrix row, or JSON contract changed after source access.

## Access Boundary

Session ID: `2f1e5ba2-ca1e-42b4-b0e1-3faa7591dde9`

The `riotbox.source_holdout_development_access_log.v3` gate opened exactly the
four registered development files in canonical order, verified each raw
SHA-256 and registered PCM format, and delivered the same bounded bytes once
to the in-process qualification owner. The access record reports:

- `access_status: completed`
- `delivery_status: completed`
- `directory_discovery_performed: false`
- `holdout_metadata_comparison.audio_files_opened: false`
- four development files opened and finalized as
  `verified_and_delivered_to_owner`
- no commercial-reference access

Holdout identity metadata was used only for the pre-open exclusion check. No
holdout audio, commercial reference, source-directory discovery, candidate
render, or playback occurred.

## Qualification Results

| Development case | Pre-NMS peaks | NMS peaks | Event-level onsets | Resolved bodies | Frozen events | Result |
| --- | ---: | ---: | ---: | ---: | ---: | --- |
| `oga_cinameng_can_be_so_beautiful` | 55 | 41 | 9 | 5 | 3 | qualified |
| `oga_marwan_cinematic_percussion` | 0 | 0 | 0 | 0 | 0 | rejected |
| `oga_william_hector_horde_war_drums` | 1 | 1 | 0 | 0 | 0 | rejected |
| `oga_frosty_ham_osdrums` | 20 | 11 | 5 | 3 | 3 | qualified |

`oga_marwan_cinematic_percussion` failed with
`insufficient_eligible_events` and `source_feature_requirements_unmet` after
the detector produced no frozen peaks. `oga_william_hector_horde_war_drums`
produced one peak, but its anatomy refused as `edge_only_impulse`; it then
failed the same source-level requirements. The overall typed refusal is
`positive_source_failed` for those two cases.

Because only two sources qualified, the analyzer emitted no pair contrasts,
no valid source partitions, and no event catalog. It did not impute missing
features or substitute fallback events.

## Local Evidence Identity

The bounded session remains local under
`/tmp/riotbox-1428-stage-a-qualification.KTklbt`. Its identity hashes are:

- session JSON:
  `0e5cf6e764330360b22752271f6b3b1e0623c280a230d07b479925346dbc84c8`
- development access log:
  `edb3ff494335221fb586890d2094dda38c6f53306b6a9aa4d9a01998332f53e1`
- qualification rejection:
  `67fff3c8b50dd17f783992d21bacde909555a99d7d90f18830c5c857eb7daa85`
- qualification commit marker:
  `6ba958ba742666513352335c32039e761797498fc0002feff344c01ebc5ad6d1`

The commit marker binds the final rejected session, access-log hash, and
rejection-artifact hash. These hashes preserve the local observation's
identity; they do not turn temporary local evidence into a release artifact.

## Stop Rule

- Do not execute the 3-family by 4-source by 2-event matrix.
- Do not render a candidate or request human playback.
- Do not tune the frozen detector, anatomy, thresholds, source roles, or
  contracts to these results.
- Any later retry requires a new versioned preregistration and decision-log
  entry under RBX-252 change control before recomputation. The current session
  and rejection cannot be reused as qualification evidence.
