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

Git commit `c60cbb392491950fdbb2edaf15a9f8926db51c71` permanently
preserves every path/byte pair covered by that aggregate. Later source-blind
hardening commits are not part of the executed snapshot and cannot be cited as
the code that produced this session.

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

The qualification owner decoded and bound PCM in Python. The separate Rust
authorization/bound-PCM scaffold was not called and its synthetic tests are
not access-path or source-binding evidence for this session.

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
produced one peak for which the frozen v1 analyzer emitted the reason code
`edge_only_impulse`; the measured fact is only that no physical-onset candidate
passed the frozen baseline, peak, signal-floor, and persistence gates. The code
does not prove an acoustically edge-only or bodyless impulse. The source then
failed the same source-level requirements. The overall typed refusal remains
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

The qualification commit marker is a local artifact marker, not a Git commit.
It binds the final rejected session, access-log hash, and rejection-artifact
hash. These hashes preserve the local observation's identity; they do not turn
temporary local evidence into a release artifact.

## Post-Execution Ultra Audit

The executed v1 analyzer loaded and byte-validated the canonical protocol, and
the aggregate snapshot confirms no covered implementation file drifted before
the Git checkpoint. Its in-memory `FrozenStageAProtocol.document` was not,
however, recursively immutable and its constructor could be forged by another
caller. No mutation occurred on the inspected runner path, so this does not
reclassify the historical fail-closed stop; it does prohibit treating that run
as evidence for the later hardened implementation.

RBX-254 also closes the v1 qualification runner with the typed refusal
`stage_a_v1_execution_closed_by_rbx_254` before session-directory validation,
contract validation, subprocess preflight, or safe-access callback. A
source-blind fixture substitutes sentinels for all four boundaries and proves
that none is reached. The executable historical body exists only at the Git
checkpoint above.

F1, F2, and F3 were never rendered on a development event because admission
stopped first. F3-v2's right-aligned envelopes and controllers are causal only
after offline whole-source DC means, anatomy, and masks are frozen; neither its
old `strict_causality_pass` field nor this session proves an end-to-end
streaming-causal transform. A future source-backed candidate F3 render must
also bind the analyzer's exact frozen means instead of silently recomputing
potentially different bits.

The next retry therefore requires Protocol v2, Prequalification v3, Impact
Role v2, Event Anatomy v2, v2 source-analysis/qualification/catalog-or-reject
schemas, Matrix v3, corrected refusal semantics, a new implementation
snapshot, and a new RBX decision before any source recomputation. Registry v3
is additionally required for the selected corpus expansion. Detector v1 and
F3-v2 may be retained only if their equations and outputs remain unchanged and
Protocol v2 states F3's conditional causal scope exactly; any changed F3
values, hashes, or output require an F3 component bump. The frozen v1 JSON and
its rejection stay unchanged as historical evidence.

The historical v1 Python analyzer and the internal Rust PCM scaffold exceed
the repository's normal file-size review range. They remain cohesive on this
rejection branch to keep the executed v1 algorithm auditable against its Git
checkpoint and to avoid a behavior-plus-module-ownership rewrite after source
results. The Protocol-v2 enabler must establish real `protocol`,
`detector_anatomy`, `source_qualification`, `authorization`, `wav_binding`, and
`render_bridge` module boundaries while preserving a narrow facade; textual
`include!` sharding is not acceptable.

## Stop Rule

- Do not execute the 3-family by 4-source by 2-event matrix.
- Do not render a candidate or request human playback.
- Do not tune the frozen detector, anatomy, thresholds, source roles, or
  contracts to these results.
- Any later retry requires the versioned Protocol-v2 boundary described above
  and a decision-log entry under RBX-252 change control before recomputation.
  The current session and rejection cannot be reused as qualification evidence.
