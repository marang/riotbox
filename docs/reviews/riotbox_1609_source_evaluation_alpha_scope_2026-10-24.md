# RIOTBOX-1609 — initial source evaluation alpha

Date: 2026-10-24. Classification: bounded release maintenance/contract enabler,
not a new audible mechanism. Base: `9254eb8fca42276e91a8471207ef67f9ab4e5d0c`
(RIOTBOX-1606 merged). Owning contract: `docs/workflow/source_evaluation_release.md`.

## Scope decision and prior assessment

There are no previous Riotbox tags/releases; workspace0.1.0 was unreleased.
Choose `0.1.0-alpha.1`, not a maintenance successor and not1.0/stage-ready.
This distributes the existing source-available prototype for local private
non-commercial evaluation without installers, qualified platform binaries,
private examples or generated listening packs.

The older RIOTBOX-1039 comment overgeneralized1.0 blockers. Roadmap§13 and
DoD§17 explicitly govern stage-ready1.0; PRD§6.2 excludes full DAW export and
polished cross-platform packaging from MVP.1038/1039/1041/1036 retain their
current larger scope; none is closed or silently redefined here. In contrast,
workflow/github_pr_ci.md's phase/release `ci-broad` requirement is not limited
to1.0 and remains a gate. The alpha label is not a waiver.

| Evidence or requirement | Valid use for this release |
| --- | --- |
| Exact-candidate `just ci`, callback tests, native CI | Technical regression proof only; merged-tree equality required for reuse |
| Authorized `just ci-broad` | Required current Development baseline, never human or Holdout proof |
| RIOTBOX-1475 six-family reconciliation including Dense | Historical bounded P023 demo context only; no current-artifact verdict transfer |
| RIOTBOX-1589 generated virtual-host result | Historical consumed result, not new host/device/endurance qualification |
| RIOTBOX-1608 dependency review and source/license checks | Reused change review; final candidate still passes current CI/vendor checks |
| RIOTBOX-1607 malformed snapshot provenance | Explicit known alpha limitation, not demonstrated ordinary Save corruption |
|1.0 host/soak/full-export/platform work | Remains open; no such claims in source-only distribution |

The release slice changes no renderer, musical recipe, source selector,
threshold, Session/replay schema, queue or `JamAppState`. Version metadata does
change the existing application-version stamp; protocol versions remain intact.

## Prospective access and package controls

No source bytes informed the decision or access policy. Static traversal of the
unchanged broad recipes identified exactly seven legacy Development paths.
The new explicit user authorization allows one broad invocation after reviewed
preflight, not directory discovery, substitution, retry or playback.

Five full-source identities are recorded in older contracts/reviews. Two have
no historical full-source SHA in inspected records; explicit enrollment reads
their exact named files once and immediately rejects protected hashes before
rendering. A Beat08 resample-tap hash was explicitly rejected as a false source
identity. Protected case/path/available-hash exclusion uses the union of tracked
registry v1/v2/v3 metadata without opening protected audio. No Stage-A protocol
or old registry is modified.

The staging helper pins metadata, rejects symlinks/hardlinks/nonregular,
oversize or changing files, freezes all seven accepted hashes before staging,
and records partial failures honestly. Only accepted bytes enter the owned
fresh validation worktree. Existing original-checkout QA outputs are preserved.

The package helper uses exact-commit Git blobs, not filesystem contents,
checks workspace/lock versions and required licenses/vendor records, rejects
private/generated/audio inputs before reading their blobs where path metadata
suffices, and creates/verifies a deterministic archive without overwrites.
The18 pre-existing22–29byte `.wav` blobs are identity-test text placeholders,
not embedded source recordings. Runtime/build and asset-download gates are
separate from these helper tests.

## Branch-review findings and disposition

- **P1, Spec And Evidence Auditor — fixed:** the first package path filter
  rejected the real production `crates/riotbox-audio/src/source_audio/*.rs`
  module. Minimal package fixtures missed that layout. Permit only that exact
  Rust-code namespace, retaining the private/data/audio veto elsewhere. Add a
  representative production-layout archive test and negative exception controls.
  Metadata-only inspection of all3487 base tracked paths then reports zero
  false rejections; no source bytes are needed for this check.
- **P2, root Test Evidence — fixed:** a generic `protected_audio_accessed:false`
  remained false even when enrollment detected a protected content-hash
  collision. Separate registered-path exclusion from hash-identity assessment;
  record collision/incomplete/all-seven outcomes explicitly. Generated success
  and first-collision fixtures assert truthful receipts and no further access.

After these corrections, the combined helper gate executes21 package tests and
23 staging tests, all passing. Inputs are generated temporary Git/text fixtures,
not real sources, sessions or audio. Independent reviews and final self-review
are reported at the exact PR boundary; pending CI/source/build/publication gates
must not be inferred from this document.

An optional bounded Jev scope sanity-check (jev-1.14.0;1054 input/102 output
tokens) supported separating1.0 aspirations from this narrower scope and keeping
the broad gate. It is neither authority nor correctness/audio qualification.

## Publication proof location

The final release manifest and PR/Linear updates bind actual candidate/merge
commits, identical tree, local/broad/build commands and log hashes, native
workflow results, source archive and downloaded checksums. These are execution
results, not predeclared passes or a self-referential source-commit hash. No
release is qualified until all owning-contract gates actually pass.
