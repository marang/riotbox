# RIOTBOX-1475 Release-Demo Evidence Reconciliation

- Date: 2026-08-26
- Partition: existing Development evidence only; no new audio qualification
- Decision: RBX-346
- Result: covered P023 release-demo scope is `release_ready`

## Scope

RIOTBOX-1475 resolves the two aggregate readiness blockers left by
RIOTBOX-1474 without changing sound, opening a new source session, rendering a
candidate, or asking for another human verdict. The immutable current live demo
bank has SHA-256
`dfd04750c010493e06f99d73139963a6666239cef78912f34235091481aa4864`.

The frozen reconciliation contract
`docs/benchmarks/release_demo_evidence_reconciliation_v1.json` has SHA-256
`3c222d13f3f19afb49b11c1a2684f5b5f3f9663b4899aca972bc5e49cf972eb8`.
It does not delete or relabel the rejected RIOTBOX-1461 scripted Dense
candidate. That entry remains visible as superseded negative evidence with its
original failure and `chop_policy` route. The later RIOTBOX-1474 exact-product
human pass is its same-source, same-family current successor.

## Quality Ownership

The v2 aggregate report accepts quality only from the complete non-fixture
product set:

- exact RuntimeMix human-pass entries for Dense, sparse, and tonal material;
- reviewed no-fallback degraded or unavailable outcomes for bad-timing,
  pad/noise, and weak sources.

The professional-output suite remains required diagnostic context and remains
`scripted_generation: true`, `quality_proof: false`. Fixture reconciliation is
covered by mutation tests but cannot produce quality proof.

The final readiness report has SHA-256
`c237313a3b8f1b39db6ba4567e014e31623efe00e129ac17cd0acadc3410cccb`.
It validates with all six family-success contracts covered, zero active
weak/fail entries, one retained superseded negative entry, zero queued reviews,
no blockers, and `release_readiness: release_ready` for this covered P023
release-demo scope.

## Broad-CI Access Incident

The normal local `just ci` closeout entered its legacy broad audio-QA layer and
reopened registered Development WAVs despite this ticket authorizing no source
audio access. The process was stopped fail-closed with SIGINT as soon as the
active registered-source generator was identified. No descendant remained; no
Holdout or commercial reference was opened; no source directory was
discovered; and no incident output or metric was used for the product or
readiness decision.

The local incident record has SHA-256
`0d03b04511c12fcfda7cfb84ed97ee8b9aa066fec6dec1bcc8c598b6ee090d7a`.
Every source-backed output root from that interrupted run is excluded from
RIOTBOX-1475 evidence. The pre-incident demo-bank, contract, and readiness
hashes remained unchanged. Per RBX-337, local closeout uses only the source-free
formatter, Rust tests, clippy, focused reconciliation/readiness fixtures, JSON,
Decision, docs, and diff gates; the broad local audio layer is recorded as
intentionally interrupted, not passed.

## Claim Boundary

This closes the expanded P023 release-demo readiness scope represented by the
current bank. It adds no new sound, source-general quality, Holdout evidence,
percussive-hardness claim, automatic arrangement, universal-quality claim, or
Riotbox 1.0 release claim.
